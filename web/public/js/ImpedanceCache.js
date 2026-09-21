// public/js/ImpedanceCache.js
/**
 * 高性能本地 LRU 缓存与持久化存储
 * 解决重复计算导致的卡顿，针对相同参数瞬间 0ms 返回
 */
export class ImpedanceCache {
  constructor(maxSize = 1000, storageKey = 'pcb_impedance_cache_v1') {
    this.maxSize = maxSize;
    this.storageKey = storageKey;
    this.cache = new Map();
    this.hits = 0;
    this.misses = 0;
    this.loadFromStorage();
  }

  /**
   * 生成规范的计算入参哈希/键
   * 包含拓扑模式、几何尺寸、介质厚度与介电常数、目标阻抗
   */
  generateKey(calcMark, arg) {
    // 提取关键参数并统一精度避免浮点微小抖动导致的缓存未命中
    const keys = [
      calcMark || '',
      Number(arg.H1 || 0).toFixed(3),
      Number(arg.Er1 || 0).toFixed(2),
      Number(arg.H2 || 0).toFixed(3),
      Number(arg.Er2 || 0).toFixed(2),
      Number(arg.T1 || 0).toFixed(3),
      Number(arg.C1 || 0).toFixed(2),
      Number(arg.C2 || 0).toFixed(2),
      Number(arg.CEr || 0).toFixed(2),
      Number(arg.Zo || 0).toFixed(1),
      arg.dCalculateMode || 3,
      Number(arg.W1 || 0).toFixed(2),
      Number(arg.S1 || 0).toFixed(2),
      Number(arg.D1 || 0).toFixed(2),
      Number(arg.W2LinkW1Incr || 0.5).toFixed(2)
    ];
    return keys.join('|');
  }

  get(calcMark, arg) {
    const key = this.generateKey(calcMark, arg);
    if (this.cache.has(key)) {
      this.hits++;
      const val = this.cache.get(key);
      // 刷新 LRU 顺序
      this.cache.delete(key);
      this.cache.set(key, val);
      return val;
    }
    this.misses++;
    return null;
  }

  set(calcMark, arg, result) {
    const key = this.generateKey(calcMark, arg);
    if (this.cache.has(key)) {
      this.cache.delete(key);
    } else if (this.cache.size >= this.maxSize) {
      // 剔除最久未使用的项（Map 的第一个键）
      const oldestKey = this.cache.keys().next().value;
      this.cache.delete(oldestKey);
    }
    this.cache.set(key, {
      result,
      timestamp: Date.now()
    });
    this.saveToStorage();
  }

  clear() {
    this.cache.clear();
    this.hits = 0;
    this.misses = 0;
    try {
      localStorage.removeItem(this.storageKey);
    } catch (e) {}
  }

  getStats() {
    const total = this.hits + this.misses;
    return {
      size: this.cache.size,
      hits: this.hits,
      misses: this.misses,
      hitRate: total > 0 ? ((this.hits / total) * 100).toFixed(1) + '%' : '0.0%'
    };
  }

  saveToStorage() {
    try {
      // 仅保留最近的 200 条存入 localStorage 避免占用过多空间
      const entries = Array.from(this.cache.entries()).slice(-200);
      localStorage.setItem(this.storageKey, JSON.stringify(entries));
    } catch (e) {
      // Ignore quota exceeded errors
    }
  }

  loadFromStorage() {
    try {
      const data = localStorage.getItem(this.storageKey);
      if (data) {
        const entries = JSON.parse(data);
        for (const [k, v] of entries) {
          this.cache.set(k, v);
        }
      }
    } catch (e) {}
  }
}
