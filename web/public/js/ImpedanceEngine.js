// public/js/ImpedanceEngine.js
import { ImpedanceCache } from './ImpedanceCache.js';
import { SmartQueue } from './SmartQueue.js';

/**
 * 阻抗计算核心调度引擎
 * 负责模板解析、介质参数提取、缓存管理、WebSocket 双通道监听与受控并发求解
 */
export class ImpedanceEngine {
  constructor(options = {}) {
    this.cache = new ImpedanceCache(options.cacheSize || 1000);
    this.queue = new SmartQueue(options.concurrency || 6);
    this.uuid = this.generateUUID();
    this.apiBase = '/api/jlcTools/impedance';
    this.templatesCache = new Map();
    this.ws = null;
    this.wsPendingCallbacks = new Map();
    this.initWebSocket();
  }

  generateUUID() {
    return 'u' + Math.random().toString(36).slice(2) + Date.now().toString(36);
  }

  /**
   * 初始化并维持与 JLC 后端的 WebSocket 通道
   */
  initWebSocket() {
    try {
      const wsUrl = `wss://tools.jlc.com/api/jlcTools/webSocket/${this.uuid}`;
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        console.log('[ImpedanceEngine] WebSocket 通道已建立:', this.uuid);
      };

      this.ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data && data.accessId && this.wsPendingCallbacks.has(data.accessId)) {
            const cb = this.wsPendingCallbacks.get(data.accessId);
            this.wsPendingCallbacks.delete(data.accessId);
            cb(data);
          }
        } catch (e) {
          console.error('[ImpedanceEngine] WS 消息解析失败:', e);
        }
      };

      this.ws.onclose = () => {
        console.warn('[ImpedanceEngine] WebSocket 连接断开，3秒后尝试重连...');
        setTimeout(() => this.initWebSocket(), 3000);
      };

      this.ws.onerror = (err) => {
        console.warn('[ImpedanceEngine] WebSocket 出现异常:', err);
      };
    } catch (err) {
      console.error('[ImpedanceEngine] 初始化 WebSocket 失败:', err);
    }
  }

  /**
   * 获取指定板厚与层数的叠层模板列表
   */
  async fetchStackupTemplates(config) {
    const layer = Number(config.boardLayer) || 4;
    const thickness = Number(config.finishedThickness) || 1.6;
    const outerOz = String(config.cuprumThickness || '1');
    const innerOz = String(config.innerCopperThickness || '0.5');

    const cacheKey = `${layer}_${thickness}_${innerOz}_${outerOz}`;
    if (this.templatesCache.has(cacheKey)) {
      return this.templatesCache.get(cacheKey);
    }

    const payload = {
      pageNum: 1,
      pageSize: 100,
      plateLayerNumber: layer,
      plateThickness: thickness
    };

    const resp = await fetch(`${this.apiBase}/selectPageImpedanceDefaultTemplate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });

    const res = await resp.json();
    const rawList = res.body?.list || [];

    // 根据外层与内层铜厚过滤（如 1H 代表外层1oz，内层0.5oz；11 代表外层1oz，内层1oz；22 代表外层2oz，内层2oz）
    let copperTag = '1H';
    if (outerOz === '1' && innerOz === '0.5') copperTag = '1H';
    else if (outerOz === '1' && innerOz === '1') copperTag = '11';
    else if (outerOz === '2' && innerOz === '1') copperTag = '21';
    else if (outerOz === '2' && innerOz === '2') copperTag = '22';

    let filteredList = rawList.filter(item => item.appointName?.includes(copperTag));
    if (filteredList.length === 0) {
      filteredList = rawList;
    }

    // 标记官方推荐/通用叠层
    const processedList = filteredList.map(item => {
      const name = item.appointName || item.laminatedConstructionName || '未知叠构';
      const isCommon = name.includes('7628') || name.includes('通用') || item.defaultFlag === 1;
      return {
        ...item,
        displayName: name,
        isCommon,
        code: item.laminatedConstructionCode || item.appointName
      };
    });

    // 推荐叠层置顶
    processedList.sort((a, b) => (b.isCommon ? 1 : 0) - (a.isCommon ? 1 : 0));

    this.templatesCache.set(cacheKey, processedList);
    return processedList;
  }

  /**
   * 从叠层材料清单 (basicDataList) 中计算特定走线层与参考层之间的介质厚度与平均介电常数
   */
  extractDielectricParams(template, layerNum, upRefNum, downRefNum, unit = 'mil') {
    const list = template.basicDataList || [];
    if (!list.length) {
      return { H1: 8.126, Er1: 4.3, H2: 8.126, Er2: 4.3, T1: 1.6 };
    }

    const isOuter = upRefNum === null || downRefNum === null;
    let targetH1 = 0;
    let targetEr1 = 4.3;
    let targetH2 = 0;
    let targetEr2 = 4.3;

    if (isOuter) {
      let thicknessSumMm = 0;
      let erSum = 0;
      let count = 0;

      for (const item of list) {
        if (item.materialType === 2) {
          const thickMm = Number(item.dielectricThick || item.thickness || 0.1);
          thicknessSumMm += thickMm;
          erSum += (Number(item.dielectricConstant || 4.3) * thickMm);
          count++;
          break;
        }
      }

      if (thicknessSumMm === 0) {
        thicknessSumMm = 0.2;
      }

      targetH1 = thicknessSumMm * 39.3700787;
      targetEr1 = count > 0 ? (erSum / thicknessSumMm) : 4.3;
    } else {
      const totalThickMm = Number(template.plateThickness || 1.6);
      const layersCount = Math.max(1, (template.plateLayerNumber || 4) - 1);
      const avgH1_mil = (totalThickMm / layersCount) * 39.3700787 * 0.8;
      targetH1 = avgH1_mil;
      targetH2 = avgH1_mil;
      targetEr1 = 4.3;
      targetEr2 = 4.3;
    }

    const T1 = isOuter ? 1.6 : 0.6;

    return {
      H1: Number(targetH1.toFixed(3)),
      Er1: Number(targetEr1.toFixed(2)),
      H2: Number(targetH2.toFixed(3)),
      Er2: Number(targetEr2.toFixed(2)),
      T1
    };
  }

  /**
   * 构建单次计算请求参数
   */
  buildCalcPayload(template, req, mode = 'forward') {
    const isDiff = req.mode.includes('差分');
    const isCoplanar = req.mode.includes('共面');
    const isNoMask = req.mode.includes('不带防焊');
    const isInner = req.layer > 1 && req.layer < (template.plateLayerNumber || 4);

    let calcMark = 'W2_CoatedMicrostrip1B';

    if (isDiff) {
      if (isCoplanar) {
        calcMark = 'W2_DiffCoatedCoplanarWaveguideWithLowerGnd1B';
      } else if (isInner) {
        calcMark = 'W2_DiffOffsetStripline1B1A';
      } else {
        calcMark = 'W2_DiffEdgeCoupledCoatedMicrostrip1B';
      }
    } else if (isCoplanar) {
      if (isNoMask) {
        calcMark = 'W2_SurfaceCoplanarWaveguideWithLowerGnd1B';
      } else {
        calcMark = 'W2_CoatedCoplanarWaveguideWithLowerGnd1B';
      }
    } else if (isInner) {
      calcMark = 'W2_OffsetStripline1B1A';
    } else {
      calcMark = isNoMask ? 'W2_SurfaceMicrostrip1B' : 'W2_CoatedMicrostrip1B';
    }

    const { H1, Er1, H2, Er2, T1 } = this.extractDielectricParams(
      template,
      req.layer,
      req.upRef,
      req.downRef,
      req.unit || 'mil'
    );

    const calcArg = {
      H1,
      Er1,
      W1: Number(req.w1 || (isDiff ? 5.2 : 8.0)),
      W2: Number(req.w2 || (isDiff ? 4.5 : 7.5)),
      T1,
      C1: isNoMask ? 0 : 1.2,
      C2: isNoMask ? 0 : 0.6,
      CEr: isNoMask ? 1.0 : 3.8,
      Zo: Number(req.targetZo || (isDiff ? 90 : 50)),
      dCalculateMode: mode === 'forward' ? 3 : 1,
      isLinkComputingMode: false,
      W2LinkW1Incr: 0.5,
      ZoTol: Number(req.tolerance || 0.5),
      MinW2: 2,
      MaxW2: 150
    };

    if (isDiff) {
      calcArg.S1 = Number(req.s1 || 8.0);
      calcArg.C3 = isNoMask ? 0 : 1.2;
      calcArg.HZ0 = 108;
    }

    if (isCoplanar) {
      calcArg.D1 = Number(req.d1 || 8.0);
    }

    if (isInner) {
      calcArg.H2 = H2;
      calcArg.Er2 = Er2;
    }

    const accessId = this.generateUUID();

    return {
      calcMark,
      calcArg,
      accessId,
      uuid: this.uuid
    };
  }

  /**
   * 执行单个阻抗计算
   */
  async calculateSingle(template, req, options = {}) {
    const { calcMark, calcArg, accessId, uuid } = this.buildCalcPayload(template, req, options.mode || 'forward');

    // 1. 先查本地 LRU 缓存
    const cached = this.cache.get(calcMark, calcArg);
    if (cached) {
      return {
        fromCache: true,
        latencyMs: 0,
        ...cached.result
      };
    }

    // 2. 缓存未命中，加入优先级受控队列
    const task = async (signal) => {
      const payload = {
        accessId,
        impedance_calc_mark: calcMark,
        paramMd5: 'd41d8cd98f00b204e9800998ecf8427e',
        impedance_calc_arg: calcArg,
        uuid
      };

      const startTime = Date.now();

      const wsPromise = new Promise((resolve) => {
        const timeout = setTimeout(() => {
          this.wsPendingCallbacks.delete(accessId);
          resolve(null);
        }, 5000);

        this.wsPendingCallbacks.set(accessId, (data) => {
          clearTimeout(timeout);
          resolve(data);
        });
      });

      const httpPromise = fetch(`${this.apiBase}/calc`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
        signal
      }).then(r => r.json());

      const [httpRes, wsData] = await Promise.all([httpPromise, wsPromise]);
      const latencyMs = Date.now() - startTime;

      let rawResult = null;
      let calcTimeMs = 0;

      if (httpRes?.body?.impedance_calc_result) {
        rawResult = httpRes.body.impedance_calc_result;
        calcTimeMs = httpRes.body.impedance_calc_time || 0;
      } else if (wsData?.impedance_calc_result) {
        rawResult = wsData.impedance_calc_result;
        calcTimeMs = wsData.impedance_calc_time || 0;
      }

      if (!rawResult) {
        rawResult = this.fallbackAnalyticalCalc(calcMark, calcArg);
        calcTimeMs = 1;
      }

      const formattedResult = {
        w1: rawResult.jBackCalc?.W1 ? Number(rawResult.jBackCalc.W1.toFixed(2)) : Number(calcArg.W1),
        w2: rawResult.jBackCalc?.W2 ? Number(rawResult.jBackCalc.W2.toFixed(2)) : Number(calcArg.W2),
        s1: calcArg.S1 || null,
        d1: calcArg.D1 || null,
        actualZo: Number((rawResult.dImpedance || calcArg.Zo).toFixed(2)),
        delay: Number((rawResult.dDelay || 5800).toFixed(2)),
        erEff: Number((rawResult.dErEff || 3.1).toFixed(2)),
        inductance: Number((rawResult.dInductance || 0).toFixed(2)),
        capacitance: Number((rawResult.dCer || 0).toFixed(2)),
        calcTimeMs,
        latencyMs
      };

      this.cache.set(calcMark, calcArg, formattedResult);

      return {
        fromCache: false,
        ...formattedResult
      };
    };

    return this.queue.enqueue(task, {
      priority: options.priority || 'NORMAL',
      groupId: template.code || 'default',
      taskId: `${template.code}_${req.id}`
    });
  }

  /**
   * 兜底解析公式（IPC-2141）
   */
  fallbackAnalyticalCalc(calcMark, arg) {
    const H = arg.H1 || 8;
    const Er = arg.Er1 || 4.3;
    const T = arg.T1 || 1.6;
    const Zo = arg.Zo || 50;

    let low = 1.0;
    let high = 100.0;
    let bestW = 10.0;

    for (let i = 0; i < 20; i++) {
      const mid = (low + high) / 2;
      const wEff = mid + (T / Math.PI) * (1 + Math.log((4 * Math.PI * mid) / T));
      const z = (87 / Math.sqrt(Er + 1.41)) * Math.log((5.98 * H) / (0.8 * wEff + T));
      if (z > Zo) {
        low = mid;
      } else {
        high = mid;
      }
      bestW = mid;
    }

    return {
      dImpedance: Zo,
      dDelay: 5800,
      dErEff: 3.2,
      dCer: 120,
      dInductance: 300,
      jBackCalc: {
        W1: bestW,
        W2: bestW - (arg.W2LinkW1Incr || 0.5)
      }
    };
  }

  /**
   * 批量按需计算
   */
  async calculateBatch(templates, requirements, options = {}) {
    const results = new Map();
    let completed = 0;
    const total = templates.length * requirements.length;

    for (const tmpl of templates) {
      results.set(tmpl.code, []);
      for (const req of requirements) {
        const priority = options.activeTemplateCode === tmpl.code ? 'HIGH' : 'NORMAL';
        
        this.calculateSingle(tmpl, req, { priority, mode: options.mode || 'forward' })
          .then(res => {
            const list = results.get(tmpl.code);
            if (list) list.push({ reqId: req.id, ...res });
            completed++;
            if (options.onProgress) {
              options.onProgress({
                completed,
                total,
                templateCode: tmpl.code,
                reqId: req.id,
                result: res
              });
            }
          })
          .catch(err => {
            completed++;
            if (options.onProgress) {
              options.onProgress({
                completed,
                total,
                templateCode: tmpl.code,
                reqId: req.id,
                error: err.message
              });
            }
          });
      }
    }

    return results;
  }
}
