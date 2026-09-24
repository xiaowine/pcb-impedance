# JLC 阻抗 API 网关（gateway.js）

外部项目复用本项目的 API 转发时"很容易请求超时"。本文件记录定位过程、实测数据与网关用法。
所有数字均来自本机实测（Node v24.19.0，上游 `tools.jlc.com`，客户端为 Node/Python 原生 HTTP）。

---

## 1. 根因：`server.js` 未设置 `keepAliveTimeout`，沿用 Node 默认 5000ms

Node HTTP 服务端默认在空闲 5s 后关闭 keep-alive 连接。使用连接池的客户端
（Java HttpClient / Go / Python requests / axios / OkHttp）在两次请求间隔 >5s 时，
会复用一条已被服务端关闭的 TCP 连接 —— 请求写进死连接后**静默挂死**（无响应，直到客户端
自身超时）或抛 `ConnectionAbortedError`。上游并不慢，网关也不报错，因此极难排查。

复现（同一网关，客户端复用同一连接，空闲 N 秒后再发一次 POST）：

| 客户端 | 空闲 | 默认 5000ms | 修复为 65000ms |
|---|---|---|---|
| 原始套接字 | 10s | **HANG > 8s（无响应）** | ok 363ms |
| 原始套接字 | 30s | **HANG > 8s** | ok 110ms |
| Python `http.client` | 10s | **ConnectionAbortedError [WinError 10053]** | OK 179ms |
| Python `http.client` | 30s | **ConnectionAbortedError** | OK 164ms |
| Python `http.client` | 60s | — | OK 280ms |
| 两者 | 2s（< 5s） | ok | ok |

**为什么高频连续压测测不出**：压测请求间隔远小于 5s，永远命中热连接。真实调用是低频突发
（改配置、用户思考、定时轮询），间隔普遍 >5s，于是"很容易超时"。

> 修复：`server.keepAliveTimeout = 65000; server.headersTimeout = 66000;`
> 直连 `tools.jlc.com` 不受影响（上游侧 keep-alive 为 65~75s），这是**网关独有**的问题。

---

## 2. 对比压测：直连 vs 网关（同一客户端、同并发、交替执行）

`/calc` 冷参、并发 6、每轮 8s：

| 路径 | 请求数 | 成功率 | p50 | p95 | 单请求线上字节 |
|---|---|---|---|---|---|
| 直连 | 600 / 654 / 664 | 100% | 72–78ms | 83–95ms | 80–339B (gzip) |
| 经网关 | 588 / 594 / 616 | 100% | 77–80ms | 92–98ms | 197–487B |

模板接口（302KB）、并发 6：

| 路径 | 请求数 | 成功率 | p50 | 单请求线上字节 |
|---|---|---|---|---|
| 直连 | 76 / 83 | 100% | 545–574ms | **13.6KB (gzip)** |
| 经网关 | 75 / 80 | 100% | 553–577ms | **303,877B（未压缩，22×）** |

并发 30：

| 接口 | 直连 p50 / p99 | 经网关 p50 / p99 |
|---|---|---|
| 模板接口 | 690 / 1079ms | 645 / 1207ms |
| `/calc` | 78 / 95ms | 83 / 119ms |

**结论**：源 API **不限流**（并发 6 连续 24s 约 1900 次 `/calc`、并发 30 全部 100% 成功）；
网关转发延迟只多 2~5ms（p50）。原实现的真实缺陷是 **gzip 被丢弃（22 倍字节放大）** 与
**全量缓冲（TTFB = 上游完整耗时）**，在跨机/跨区链路上放大为可感知的超时。

---

## 3. 源 API 的真实约束（不可改，只能适配）

1. **`/calc` 是异步接口**：冷参数首次 POST 只返回 `{"body":null,"success":true}`，
   真正结果 0.6~2.5s 后才出现 —— 或经 WebSocket 回推（`wss://<origin>/api/jlcTools/webSocket/{uuid}`），
   或对同一参数重复 POST 后由 HTTP 内联返回。

   | 拓扑 | WS 回推 | 无 WS 时轮询取回 |
   |---|---|---|
   | 外层单端微带 | 637ms | 第 3 次 (486ms) |
   | 外层差分微带 | 818ms | 第 4 次 (791ms) |
   | 内层单端带状线 | 637ms | 第 3 次 (486ms) |
   | 内层差分带状线 | 818ms | 第 4 次 (809ms) |
   | 共面单端 | 707ms | 第 4 次 (814ms) |
   | **共面差分** | **2505ms** | 第 7 次 (**1847ms**) |
   | 重复调用（已热） | — | 内联直接返回，92–111ms |

2. **`uuid` 是强制字段**，缺失直接回 `400 必须的入参不能为空或方法入参不符合基本要求`；
   `paramMd5` 可选（参考实现固定送空串的 MD5）。
3. WS 报文含 `impedance_calc_status` / `impedance_calc_error_msg`，成功时为 `0` / `""`。
4. **Node 的 `fetch`(undici) 会自动解压响应体，但保留 `content-encoding` 头** ——
   任何"透传压缩头"的实现都会让客户端拿到"标着 gzip 的明文"。

---

## 4. gateway.js 的能力与实测

| 能力 | 实现 | 实测 |
|---|---|---|
| 流式 + 压缩 | 上游固定 gzip 省流量；客户端侧按 `Accept-Encoding` 本地重压（含 `q=0` 语义） | TTFB **115ms vs 总 1095ms**；模板接口 **303,877B → 16,706B（18 倍）**，解压校验 20 条数据正确 |
| 断开联动 | `res.on('close')` → `AbortController.abort()` | 客户端 1.5s 断开，上游 **1509ms** 看到 close（原实现跑满 20,004ms） |
| 超时 + 重试 | `AbortSignal.any([客户端信号, timeout])`；502/503/504/429 与网络错误退避重试 | 上游前两次 502 → **200 + `X-Gateway-Attempts: 3`**；上游挂死 6.8s 后明确 502 |
| `/calc` 同步化 | 自持 WS 通道按 `accessId` 关联 → 轮询兜底 → 否则 504 | 冷参 **631ms 返回真实结果**、热参 107ms、WS 不可用走轮询 978ms、**10 并发冷参无串号** |
| 入参兜底 | 自动补齐上游强制的 `uuid`（及 `paramMd5`） | 缺 `uuid` 的请求不再 400 |
| 可观测 | `/health`、`X-Gateway-Source: ws\|http\|poll`、`X-Gateway-Attempts`、`X-Gateway-Latency-Ms` | — |

失败语义：网关**不会**返回编造值（前端原实现会在 WS 超时后回填 `actualZo = targetZo` 的
IPC-2141 估算值，界面上显示为"完美结果"）；取不到结果时明确返回 `504`。

---

## 5. 用法

```bash
node gateway.js                      # 默认 3000 端口，代理 https://tools.jlc.com
PORT=8080 node gateway.js            # 自定义端口
```

| 环境变量 | 默认 | 说明 |
|---|---|---|
| `PORT` | `3000` | 监听端口 |
| `TARGET_ORIGIN` | `https://tools.jlc.com` | 上游地址（同时推导 WS 地址） |
| `UPSTREAM_TIMEOUT_MS` | `15000` | 单次上游请求超时 |
| `MAX_RETRIES` | `2` | 重试次数（退避 200ms × 3ⁿ） |
| `SYNC_DEADLINE_MS` | `12000` | `/calc` 同步等待上限，超时回 504 |
| `WS_WAIT_MS` | `6000` | WS 回推等待上限，之后转轮询 |
| `POLL_INTERVAL_MS` | `300` | 轮询间隔 |
| `GATEWAY_FORCE_GZIP` | — | `1` = 对未声明 `Accept-Encoding` 的客户端强制 gzip（RFC 7231 允许，需客户端自行解压） |

接口：

```
POST /api/jlcTools/impedance/calc                                # 同步返回结果（?raw=1 退回原始异步透传）
POST /api/jlcTools/impedance/selectPageImpedanceDefaultTemplate  # 模板列表
GET  /health                                                     # {ok, upstream, wsChannel, pendingCalc, uptimeSec}
```

客户端建议：
- 带上 `Accept-Encoding: gzip`（省 18 倍流量）；不带时默认给 identity，避免 Java `HttpURLConnection`
  这类不会自动解压的客户端拿到乱码。
- 请求体沿用上游原生格式即可，`uuid` / `paramMd5` 可省略，网关会补。
- 同步 `/calc` 的合理超时预算 ≥ 3s（共面差分最慢实测 2.5s）。

---

## 6. 与 server.js 的关系

`server.js` 仍服务于自带前端页面（静态资源 + 代理），本次仅补上 keep-alive 两行；
`gateway.js` 是面向外部项目的独立进程（不含静态资源），两者互不影响，可同时运行。
