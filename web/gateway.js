// gateway.js - 面向外部项目的 JLC 阻抗 API 网关（零依赖，Node 20+）
//
// 与 server.js（前端自带代理）的区别，本文件专门解决"外部项目接入后容易请求超时"：
//   1. 流式转发 + 压缩：不再把上游 gzip 解压后裸发（模板接口 13.6KB → 303KB，22 倍放大）。
//      上游侧固定 gzip 省流量，客户端侧按 Accept-Encoding 在本地重新压缩，且 TTFB = 上游首包
//      而非全量下载完成。（注意：Node 的 fetch 会自动解压响应体却保留 content-encoding 头，
//      所以压缩头必须由网关重新声明，不能透传。）
//   2. 断开联动：客户端超时断开时立刻中止上游请求（原实现会留下跑满 20s 的僵尸请求）。
//   3. 上游超时 + 重试：默认 15s 超时，502/503/504/429 与网络错误退避重试 2 次。
//   4. /calc 同步化：上游 /calc 是异步接口（冷参数首次 POST 只回 body:null，结果 0.6~2.5s 后
//      经 WebSocket 回推）。网关自己维持 WS 通道并按 accessId 关联，对外表现为"一次 POST 直接
//      拿到阻抗结果"，取不到则明确返回 504，绝不返回编造值。
//
// 启动：
//   node gateway.js                       # 默认 3000 端口，代理 https://tools.jlc.com
//   PORT=8080 TARGET_ORIGIN=https://tools.jlc.com node gateway.js
//
// 外部项目调用：
//   POST http://<host>:<port>/api/jlcTools/impedance/selectPageImpedanceDefaultTemplate   # 模板列表
//   POST http://<host>:<port>/api/jlcTools/impedance/calc                                # 同步拿到结果
//   GET  http://<host>:<port>/health                                                     # 健康检查
//   建议客户端带上 Accept-Encoding: gzip（可省 18 倍流量）；请求体与上游原生格式完全一致，
//   其中上游强制的 uuid（以及可选的 paramMd5）若未提供，网关会自动补齐（否则上游直接回 400）。
//
// 环境变量：PORT / TARGET_ORIGIN / UPSTREAM_TIMEOUT_MS / MAX_RETRIES / SYNC_DEADLINE_MS /
//           WS_WAIT_MS / POLL_INTERVAL_MS / GATEWAY_FORCE_GZIP=1（对未声明 Accept-Encoding 的客户端强制 gzip）

'use strict';

const http = require('node:http');
const zlib = require('node:zlib');
const { Readable } = require('node:stream');

const PORT = Number(process.env.PORT || 3000);
const TARGET_ORIGIN = (process.env.TARGET_ORIGIN || 'https://tools.jlc.com').replace(/\/+$/, '');
const WS_ORIGIN = TARGET_ORIGIN.replace(/^http/, 'ws');
const UPSTREAM_TIMEOUT_MS = Number(process.env.UPSTREAM_TIMEOUT_MS || 15000);
const MAX_RETRIES = Number(process.env.MAX_RETRIES || 2);
const RETRY_BASE_MS = Number(process.env.RETRY_BASE_MS || 200);
const SYNC_DEADLINE_MS = Number(process.env.SYNC_DEADLINE_MS || 12000);
const WS_WAIT_MS = Number(process.env.WS_WAIT_MS || 6000);
const WS_WAIT_WHEN_DOWN_MS = 600;
const POLL_INTERVAL_MS = Number(process.env.POLL_INTERVAL_MS || 300);
const FORCE_GZIP = process.env.GATEWAY_FORCE_GZIP === '1';

const CALC_PATH = '/api/jlcTools/impedance/calc';
const EMPTY_MD5 = 'd41d8cd98f00b204e9800998ecf8427e';
const RETRY_STATUS = new Set([429, 502, 503, 504]);
const HOP_BY_HOP = new Set([
  'connection', 'keep-alive', 'proxy-authenticate', 'proxy-authorization',
  'te', 'trailer', 'transfer-encoding', 'upgrade', 'host'
]);

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

function sendJson(res, status, obj, extraHeaders = {}) {
  const buf = Buffer.from(JSON.stringify(obj));
  res.writeHead(status, {
    'Content-Type': 'application/json; charset=utf-8',
    'Content-Length': buf.length,
    ...extraHeaders
  });
  res.end(buf);
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    const chunks = [];
    req.on('data', (c) => chunks.push(c));
    req.on('end', () => resolve(Buffer.concat(chunks)));
    req.on('error', reject);
  });
}

/**
 * 上游强制校验 uuid（缺了直接回 400「必须的入参不能为空或方法入参不符合基本要求」），
 * paramMd5 可选但参考实现都会带。外部项目很容易漏，这里统一补齐，不改动其它字段。
 */
function normalizeCalcBody(buf, uuid) {
  let json;
  try { json = JSON.parse(buf.toString('utf8')); } catch { return buf; }
  if (!json || typeof json !== 'object' || Array.isArray(json)) return buf;
  if (!json.impedance_calc_mark) return buf;
  let changed = false;
  if (!json.uuid) { json.uuid = uuid; changed = true; }
  if (!json.paramMd5) { json.paramMd5 = EMPTY_MD5; changed = true; }
  return changed ? Buffer.from(JSON.stringify(json)) : buf;
}

/** 转发给上游的请求头：补齐上游要求的来源标识，其余按白名单透传 */
function upstreamHeaders(req, bodyLength) {
  const headers = {
    'Content-Type': req.headers['content-type'] || 'application/json',
    'Origin': TARGET_ORIGIN,
    'Referer': `${TARGET_ORIGIN}/jlcTools/index.html`,
    'User-Agent': req.headers['user-agent'] || 'Mozilla/5.0',
    // 上游侧固定要 gzip：省网关↔上游的流量。Node 的 fetch(undici) 会自动解压，
    // 但对客户端侧的压缩由本网关重新决定（见 clientWantsGzip），不能依赖上游响应头透传。
    'Accept-Encoding': 'gzip'
  };
  if (req.headers.accept) headers['Accept'] = req.headers.accept;
  if (bodyLength > 0) headers['Content-Length'] = String(bodyLength);
  return headers;
}

/**
 * 客户端是否接受 gzip。未声明 Accept-Encoding 时按 identity 处理
 * （Java HttpURLConnection 之类客户端不会自动解压，强推 gzip 会拿到乱码）；
 * 需要对这些客户端强制压缩时设置 GATEWAY_FORCE_GZIP=1（RFC 7231 允许，但需客户端自行解压）。
 */
function clientWantsGzip(req) {
  const ae = req.headers['accept-encoding'];
  if (!ae) return FORCE_GZIP;
  for (const part of ae.toLowerCase().split(',')) {
    const [coding, ...params] = part.trim().split(';').map((s) => s.trim());
    if (params.includes('q=0')) continue;
    if (coding === 'gzip' || coding === 'x-gzip' || coding === '*') return true;
  }
  return false;
}

/**
 * 上游响应头透传：剔除逐跳头，并丢弃 content-length / content-encoding。
 * undici 的 fetch 会自动解压响应体却保留 content-encoding 头，直接透传会让客户端
 * 拿着"标着 gzip 的明文"去解压而失败；content-length 同理已与解压后的字节数不符。
 * 因此响应体统一按 chunked 下发，压缩头由本网关按实际动作重新声明。
 */
function downstreamHeaders(resp) {
  const out = {};
  for (const [k, v] of resp.headers) {
    const key = k.toLowerCase();
    if (HOP_BY_HOP.has(key) || key === 'content-length' || key === 'content-encoding') continue;
    out[k] = v;
  }
  return out;
}

/** 带超时与退避重试的上游请求；客户端断开（clientSignal）时不重试 */
async function fetchUpstream(url, { clientSignal, ...init }) {
  let lastErr = null;
  for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
    const signal = AbortSignal.any([clientSignal, AbortSignal.timeout(UPSTREAM_TIMEOUT_MS)]);
    try {
      const resp = await fetch(url, { ...init, signal });
      if (RETRY_STATUS.has(resp.status) && attempt < MAX_RETRIES) {
        await resp.body?.cancel().catch(() => {});
        lastErr = new Error(`upstream ${resp.status}`);
        console.warn(`[gateway] 上游 ${resp.status}，${RETRY_BASE_MS * 3 ** attempt}ms 后重试 (${attempt + 1}/${MAX_RETRIES})`);
        await sleep(RETRY_BASE_MS * 3 ** attempt);
        continue;
      }
      return { resp, attempts: attempt + 1 };
    } catch (err) {
      lastErr = err;
      if (clientSignal.aborted) throw err;               // 客户端已放弃，不必重试
      if (attempt < MAX_RETRIES) {
        console.warn(`[gateway] 上游请求失败 ${err.name}: ${err.message}，${RETRY_BASE_MS * 3 ** attempt}ms 后重试`);
        await sleep(RETRY_BASE_MS * 3 ** attempt);
        continue;
      }
      throw err;
    }
  }
  throw lastErr || new Error('upstream unavailable');
}

/**
 * 流式转发上游响应：先发响应头再边收边发；按客户端能力在本地重新压缩。
 * 上游响应体已被 undici 解压，因此这里下发的一定是明文，压缩与否由本网关决定。
 */
async function pipeUpstream(req, res, resp) {
  const gzip = clientWantsGzip(req);
  const headers = downstreamHeaders(resp);
  if (gzip) headers['Content-Encoding'] = 'gzip';

  if (!resp.body || req.method === 'HEAD') {
    res.writeHead(resp.status, headers);
    res.end();
    return;
  }

  res.writeHead(resp.status, headers);
  let body = Readable.fromWeb(resp.body);
  if (gzip) body = body.pipe(zlib.createGzip({ level: zlib.constants.Z_BEST_SPEED }));
  body.on('error', (err) => {
    console.warn('[gateway] 上游响应流中断:', err.message);
    res.destroy(err);                                    // 截断必须让客户端可见，不能装作正常结束
  });
  res.on('close', () => body.destroy());
  body.pipe(res);
}

/**
 * 结果通道：网关自己维持与上游的 WebSocket，按 accessId 关联 /calc 的异步结果。
 * 上游 /calc 的 HTTP 响应只有回执（body:null），真正的结果从这条通道回推。
 */
class ResultChannel {
  constructor(origin, uuid) {
    this.url = `${origin}/api/jlcTools/webSocket/${uuid}`;
    this.uuid = uuid;
    this.state = 'idle';
    this.pending = new Map();
    this.failures = 0;
    this.connect();
  }

  connect() {
    if (typeof WebSocket === 'undefined') {
      this.state = 'unsupported';
      console.warn('[gateway] 当前 Node 无内置 WebSocket，/calc 将退化为轮询模式（建议 Node 22+）');
      return;
    }
    this.state = 'connecting';
    let ws;
    try {
      ws = new WebSocket(this.url);
    } catch (err) {
      this.state = 'error';
      this.scheduleReconnect();
      return;
    }
    this.ws = ws;
    ws.addEventListener('open', () => {
      this.state = 'open';
      this.failures = 0;
      console.log('[gateway] WS 结果通道已建立:', this.uuid);
    });
    ws.addEventListener('message', (ev) => {
      let data;
      try { data = JSON.parse(ev.data); } catch { return; }
      const waiter = this.pending.get(data.accessId);
      if (!waiter) return;
      this.pending.delete(data.accessId);
      clearTimeout(waiter.timer);
      waiter.resolve(data);
    });
    ws.addEventListener('close', () => {
      this.state = 'closed';
      this.failAll();
      this.scheduleReconnect();
    });
    ws.addEventListener('error', () => { this.state = 'error'; });
  }

  scheduleReconnect() {
    if (this.reconnectTimer) return;
    const delay = Math.min(30000, 1000 * 2 ** this.failures++);
    this.reconnectTimer = setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, delay);
    console.warn(`[gateway] WS 通道断开，${delay}ms 后重连`);
  }

  /** 登记一次等待；超时或通道断开时返回 null，由调用方转入轮询兜底 */
  next(accessId, timeoutMs) {
    return new Promise((resolve) => {
      const timer = setTimeout(() => {
        this.pending.delete(accessId);
        resolve(null);
      }, timeoutMs);
      this.pending.set(accessId, { resolve, timer });
    });
  }

  failAll() {
    for (const [id, waiter] of this.pending) {
      clearTimeout(waiter.timer);
      this.pending.delete(id);
      waiter.resolve(null);
    }
  }
}

let accessSeq = 0;
const nextAccessId = () => `gw-${Date.now().toString(36)}-${++accessSeq}`;

/**
 * /calc 同步化：POST 上游 → 等待 WS 回推 / HTTP 内联结果 → 兜底轮询 → 明确失败。
 * 返回体保持上游原始结构，仅把 body.impedance_calc_result 填充为真实结果。
 */
async function handleCalcSync(req, res, url, channel, clientSignal) {
  let payload;
  try {
    payload = JSON.parse((await readBody(req)).toString('utf8'));
  } catch {
    return sendJson(res, 400, { error: 'Bad Request', message: '请求体必须是合法 JSON' });
  }
  if (!payload || typeof payload !== 'object' || !payload.impedance_calc_mark) {
    return sendJson(res, 400, { error: 'Bad Request', message: '缺少 impedance_calc_mark' });
  }

  const upstreamUrl = TARGET_ORIGIN + url.pathname + url.search;
  const callerAccessId = payload.accessId || null;
  payload.paramMd5 = payload.paramMd5 || EMPTY_MD5;

  const post = async (accessId) => {
    const body = Buffer.from(JSON.stringify({ ...payload, accessId, uuid: channel.uuid }));
    const { resp, attempts } = await fetchUpstream(upstreamUrl, {
      method: 'POST',
      headers: upstreamHeaders(req, body.length),
      body,
      clientSignal
    });
    const text = await resp.text();
    let json = null;
    try { json = JSON.parse(text); } catch { /* 上游非 JSON（如 502 HTML）由下方统一报错 */ }
    return {
      attempts,
      status: resp.status,
      json,
      result: json?.body?.impedance_calc_result || null,
      calcTime: json?.body?.impedance_calc_time ?? null
    };
  };

  const started = Date.now();
  const accessId = nextAccessId();
  // 先登记 WS 等待再发请求，避免结果比登记更早到达而丢失
  const wsWait = channel.next(accessId, channel.state === 'open' ? WS_WAIT_MS : WS_WAIT_WHEN_DOWN_MS);

  let first;
  try {
    first = await post(accessId);
  } catch (err) {
    if (clientSignal.aborted) return;
    return sendJson(res, 502, { error: 'Bad Gateway', message: `上游请求失败: ${err.message}` });
  }

  let result = first.result;
  let calcTime = first.calcTime;
  let source = 'http';
  let wsData = null;

  if (!result) {
    wsData = await wsWait;
    if (wsData?.impedance_calc_result) {
      result = wsData.impedance_calc_result;
      calcTime = wsData.impedance_calc_time ?? calcTime;
      source = 'ws';
    }
  }

  // 上游报错（原先被前端忽略的字段）直接如实返回，不再伪装成计算结果
  const upstreamStatus = wsData?.impedance_calc_status ?? first.json?.body?.impedance_calc_status;
  const upstreamError = wsData?.impedance_calc_error_msg || first.json?.body?.impedance_calc_error_msg;
  if (!result && (upstreamError || (upstreamStatus !== undefined && upstreamStatus !== 0))) {
    return sendJson(res, 502, {
      error: 'Upstream Calc Error',
      status: upstreamStatus,
      message: upstreamError || '上游计算失败'
    });
  }

  // 兜底轮询：上游按参数缓存结果，重复 POST 会在 ~0.5~2s 后内联返回（无需 WS 通道）
  const deadline = started + SYNC_DEADLINE_MS;
  while (!result && Date.now() < deadline) {
    if (clientSignal.aborted) return;
    await sleep(POLL_INTERVAL_MS);
    try {
      const again = await post(nextAccessId());
      if (again.result) {
        result = again.result;
        calcTime = again.calcTime ?? calcTime;
        source = 'poll';
        break;
      }
    } catch (err) {
      if (clientSignal.aborted) return;
      console.warn('[gateway] 轮询失败:', err.message);
    }
  }

  if (!result) {
    return sendJson(res, 504, {
      error: 'Gateway Timeout',
      message: `上游在 ${SYNC_DEADLINE_MS}ms 内未返回计算结果（已尝试 WS 回推与轮询）`
    }, { 'X-Gateway-Source': source });
  }

  const base = first.json && typeof first.json === 'object' ? first.json : { message: null, result: 'success', success: true };
  sendJson(res, 200, {
    ...base,
    accessId: callerAccessId ?? accessId,
    body: { ...(base.body || {}), impedance_calc_result: result, impedance_calc_time: calcTime ?? 0 }
  }, {
    'X-Gateway-Source': source,
    'X-Gateway-Attempts': String(first.attempts),
    'X-Gateway-Latency-Ms': String(Date.now() - started)
  });
}

const channel = new ResultChannel(WS_ORIGIN, `gw${Math.random().toString(36).slice(2)}${Date.now().toString(36)}`);

const server = http.createServer(async (req, res) => {
  // 客户端断开（对端超时/取消）→ 立刻中止上游请求，避免僵尸请求堆积
  const client = new AbortController();
  res.on('close', () => {
    if (!res.writableEnded) client.abort();
  });

  const url = new URL(req.url, `http://${req.headers.host || 'localhost'}`);

  if (req.method === 'OPTIONS') {
    res.writeHead(204, {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': '*',
      'Access-Control-Max-Age': '86400'
    });
    res.end();
    return;
  }

  res.setHeader('Access-Control-Allow-Origin', '*');

  if (url.pathname === '/health') {
    return sendJson(res, 200, {
      ok: true,
      upstream: TARGET_ORIGIN,
      wsChannel: channel.state,
      pendingCalc: channel.pending.size,
      uptimeSec: Math.round(process.uptime())
    });
  }

  if (!url.pathname.startsWith('/api/')) {
    return sendJson(res, 404, { error: 'Not Found', message: '网关仅处理 /api/* 与 /health' });
  }

  if (url.pathname === CALC_PATH && req.method === 'POST' && url.searchParams.get('raw') !== '1') {
    return handleCalcSync(req, res, url, channel, client.signal);
  }

  // 其余接口：流式透传 + 超时 + 重试 + 断开联动
  let body = null;
  if (req.method !== 'GET' && req.method !== 'HEAD') {
    try {
      body = normalizeCalcBody(await readBody(req), channel.uuid);
    } catch {
      return sendJson(res, 400, { error: 'Bad Request', message: '读取请求体失败' });
    }
  }

  try {
    const { resp, attempts } = await fetchUpstream(TARGET_ORIGIN + url.pathname + url.search, {
      method: req.method,
      headers: upstreamHeaders(req, body ? body.length : 0),
      body,
      clientSignal: client.signal
    });
    if (client.signal.aborted) return resp.body?.cancel().catch(() => {});
    res.setHeader('X-Gateway-Attempts', String(attempts));
    await pipeUpstream(req, res, resp);
  } catch (err) {
    if (client.signal.aborted) return;
    console.error('[gateway] 代理失败:', err);
    sendJson(res, 502, { error: 'Bad Gateway', message: err.message });
  }
});

// 与 server.js 同样的 keep-alive 语义：Node 默认 5s 会让连接池客户端复用死连接而超时
server.keepAliveTimeout = 65000;
server.headersTimeout = 66000;

server.listen(PORT, () => {
  console.log(`[JLC Impedance Gateway] http://localhost:${PORT}  ->  ${TARGET_ORIGIN}`);
  console.log(`[JLC Impedance Gateway] 同步计算: POST ${CALC_PATH}  (加 ?raw=1 退回原始异步透传)`);
});
