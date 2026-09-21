// public/js/SmartQueue.js
/**
 * 智能异步请求队列与并发控制器
 * 彻底消除浏览器连接池排队与瞬间爆发的请求拥塞
 */
export class SmartQueue {
  constructor(concurrency = 4) {
    this.concurrency = concurrency;
    this.queue = [];
    this.running = new Set();
    this.abortControllers = new Map();
    this.metrics = {
      totalCompleted: 0,
      totalFailed: 0,
      latencies: [],
      avgLatencyMs: 0
    };
    this.onMetricsChange = null;
  }

  /**
   * 添加任务到队列
   * @param {Function} taskAsync 异步执行函数 (signal) => Promise<any>
   * @param {Object} options { priority: 'HIGH' | 'NORMAL' | 'LOW', taskId: string, groupId: string }
   */
  enqueue(taskAsync, options = {}) {
    const priorityWeight = {
      HIGH: 3,
      NORMAL: 2,
      LOW: 1
    };

    const task = {
      id: options.taskId || Math.random().toString(36).slice(2),
      groupId: options.groupId || 'default',
      priority: options.priority || 'NORMAL',
      weight: priorityWeight[options.priority || 'NORMAL'],
      taskAsync,
      resolve: null,
      reject: null,
      startTime: 0,
      createdAt: Date.now()
    };

    const promise = new Promise((resolve, reject) => {
      task.resolve = resolve;
      task.reject = reject;
    });

    this.queue.push(task);
    // 按优先级降序排序，相同优先级按先来后到
    this.queue.sort((a, b) => b.weight - a.weight || a.createdAt - b.createdAt);

    this.processNext();
    return promise;
  }

  /**
   * 取消特定组或所有任务
   */
  cancelGroup(groupId) {
    // 1. 从排队队列中移除
    this.queue = this.queue.filter(task => {
      if (task.groupId === groupId) {
        task.reject(new DOMException('Aborted by user', 'AbortError'));
        return false;
      }
      return true;
    });

    // 2. 中止正在运行的任务
    for (const task of this.running) {
      if (task.groupId === groupId) {
        const controller = this.abortControllers.get(task.id);
        if (controller) {
          controller.abort();
          this.abortControllers.delete(task.id);
        }
      }
    }
  }

  cancelAll() {
    for (const task of this.queue) {
      task.reject(new DOMException('Aborted by user', 'AbortError'));
    }
    this.queue = [];

    for (const controller of this.abortControllers.values()) {
      controller.abort();
    }
    this.abortControllers.clear();
  }

  async processNext() {
    if (this.running.size >= this.concurrency || this.queue.length === 0) {
      return;
    }

    const task = this.queue.shift();
    if (!task) return;

    this.running.add(task);
    const controller = new AbortController();
    this.abortControllers.set(task.id, controller);
    task.startTime = Date.now();

    this.notifyMetrics();

    try {
      const result = await task.taskAsync(controller.signal);
      const latency = Date.now() - task.startTime;
      this.recordLatency(latency);
      this.metrics.totalCompleted++;
      task.resolve(result);
    } catch (err) {
      if (err.name === 'AbortError') {
        task.reject(err);
      } else {
        this.metrics.totalFailed++;
        task.reject(err);
      }
    } finally {
      this.running.delete(task);
      this.abortControllers.delete(task.id);
      this.notifyMetrics();
      this.processNext();
    }
  }

  recordLatency(ms) {
    this.metrics.latencies.push(ms);
    if (this.metrics.latencies.length > 50) {
      this.metrics.latencies.shift();
    }
    const sum = this.metrics.latencies.reduce((a, b) => a + b, 0);
    this.metrics.avgLatencyMs = Math.round(sum / this.metrics.latencies.length);
  }

  notifyMetrics() {
    if (typeof this.onMetricsChange === 'function') {
      this.onMetricsChange({
        running: this.running.size,
        queued: this.queue.length,
        completed: this.metrics.totalCompleted,
        failed: this.metrics.totalFailed,
        avgLatencyMs: this.metrics.avgLatencyMs
      });
    }
  }

  getStats() {
    return {
      running: this.running.size,
      queued: this.queue.length,
      completed: this.metrics.totalCompleted,
      failed: this.metrics.totalFailed,
      avgLatencyMs: this.metrics.avgLatencyMs
    };
  }
}
