// public/app.js
import { createApp, ref, reactive, computed, onMounted } from 'https://cdn.jsdelivr.net/npm/vue@3/dist/vue.esm-browser.prod.js';
import { ImpedanceEngine } from './js/ImpedanceEngine.js';
import { StackupVisualizer } from './js/StackupVisualizer.js';

const app = createApp({
  setup() {
    const engine = new ImpedanceEngine({ concurrency: 6 });

    // 1. 基础配置参数
    const form = reactive({
      plateType: '硬板',
      boardLayer: 4,
      finishedThickness: '1.6',
      cuprumThickness: '1',
      innerCopperThickness: '0.5',
      unit: 'mil'
    });

    // 2. 多阻抗需求规格列表
    const requirements = ref([
      {
        id: 'req_1',
        targetZo: 50,
        mode: '单端阻抗（外层）',
        layer: 1,
        upRef: null,
        downRef: 2,
        tolerance: 0.5,
        w1: 8,
        s1: null,
        d1: null
      },
      {
        id: 'req_2',
        targetZo: 90,
        mode: '差分阻抗（外层）',
        layer: 1,
        upRef: null,
        downRef: 2,
        tolerance: 0.5,
        w1: 4.5,
        s1: 5.5,
        d1: null
      }
    ]);

    // 3. 叠层模板列表
    const templates = ref([]);
    const calculating = ref(false);
    const calcStrategy = ref('RECOMMENDED_ONLY'); // RECOMMENDED_ONLY | ALL_BOUNDED

    // 4. 结果映射表 templateCode -> Array of results
    const resultsMap = reactive(new Map());

    // 5. 性能与队列统计
    const cacheStats = ref({ hits: 0, misses: 0, hitRate: '0.0%' });
    const queueStats = ref({ running: 0, queued: 0, completed: 0, avgLatencyMs: 0 });

    engine.queue.onMetricsChange = (metrics) => {
      queueStats.value = metrics;
      cacheStats.value = engine.cache.getStats();
    };

    // 可选层数列表
    const availableLayers = computed(() => {
      const count = form.boardLayer || 4;
      const list = [];
      for (let i = 1; i <= count; i++) {
        list.push(i);
      }
      return list;
    });

    // 选中的置顶叠构
    const pinnedTemplateCode = ref(null);

    const pinTemplate = (code) => {
      if (pinnedTemplateCode.value === code) {
        pinnedTemplateCode.value = null;
      } else {
        pinnedTemplateCode.value = code;
      }
    };

    // 过滤展示的叠层模板 (置顶优先)
    const filteredTemplates = computed(() => {
      const list = [...templates.value];
      if (pinnedTemplateCode.value) {
        const idx = list.findIndex(t => t.code === pinnedTemplateCode.value);
        if (idx !== -1) {
          const [pinned] = list.splice(idx, 1);
          list.unshift(pinned);
        }
      }
      return list;
    });
    // 获取特定叠层的计算结果列表
    const getTemplateResults = (templateCode) => {
      return resultsMap.get(templateCode) || requirements.value.map(req => ({
        reqId: req.id,
        targetZo: req.targetZo,
        mode: req.mode,
        layer: req.layer,
        upRef: req.upRef,
        downRef: req.downRef,
        tolerance: req.tolerance,
        loading: false
      }));
    };

    // 加载叠层模板
    const loadTemplates = async () => {
      try {
        templates.value = [];
        resultsMap.clear();
        const list = await engine.fetchStackupTemplates(form);
        templates.value = list.map(item => ({
          ...item,
          expanded: false, // 默认收起剖面图与材料！
          calculating: false
        }));

        // 自动触发首次计算（默认仅计算推荐叠构）
        await triggerCalculation();
      } catch (err) {
        console.error('加载叠构模板失败:', err);
      }
    };

    // 触发阻抗计算
    const triggerCalculation = async () => {
      if (!templates.value.length) return;
      calculating.value = true;

      // 决定计算的目标叠层
      const targetTemplates = calcStrategy.value === 'RECOMMENDED_ONLY'
        ? templates.value.filter(t => t.isCommon).slice(0, 2)
        : templates.value;

      // 初始化占位结果
      for (const tmpl of targetTemplates) {
        const initialResults = requirements.value.map(req => ({
          reqId: req.id,
          targetZo: req.targetZo,
          mode: req.mode,
          layer: req.layer,
          upRef: req.upRef,
          downRef: req.downRef,
          tolerance: req.tolerance,
          loading: true
        }));
        resultsMap.set(tmpl.code, initialResults);
      }

      try {
        // 执行并发/缓存调度计算
        await engine.calculateBatch(targetTemplates, requirements.value, {
          activeTemplateCode: targetTemplates[0]?.code,
          onProgress: ({ templateCode, reqId, result, error }) => {
            const list = resultsMap.get(templateCode);
            if (list) {
              const item = list.find(r => r.reqId === reqId);
              if (item) {
                if (result) {
                  Object.assign(item, result, { loading: false });
                } else {
                  item.loading = false;
                  item.error = error;
                }
              }
            }
            cacheStats.value = engine.cache.getStats();
          }
        });
      } finally {
        calculating.value = false;
      }
    };

    // 单独计算特定叠层
    const calculateSpecificTemplate = async (tmpl) => {
      tmpl.calculating = true;
      const initialResults = requirements.value.map(req => ({
        reqId: req.id,
        targetZo: req.targetZo,
        mode: req.mode,
        layer: req.layer,
        upRef: req.upRef,
        downRef: req.downRef,
        tolerance: req.tolerance,
        loading: true
      }));
      resultsMap.set(tmpl.code, initialResults);

      try {
        for (const req of requirements.value) {
          const res = await engine.calculateSingle(tmpl, req, { priority: 'HIGH' });
          const list = resultsMap.get(tmpl.code);
          const item = list.find(r => r.reqId === req.id);
          if (item) {
            Object.assign(item, res, { loading: false });
          }
        }
      } finally {
        tmpl.calculating = false;
        cacheStats.value = engine.cache.getStats();
      }
    };

    // 视口动态加载与计算调度 (对齐 Android RecyclerView 动态绑定)
    const cardObservers = new Map();

    const setupCardObserver = (el, tmpl) => {
      if (!el || cardObservers.has(tmpl.code)) return;

      const observer = new IntersectionObserver((entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            // 当卡片滑动到视口可见范围（或预加载缓冲150px）时按需触发计算！
            const results = resultsMap.get(tmpl.code);
            const needsCalc = !results || results.some(r => r.actualZo === undefined);
            if (needsCalc && !tmpl.calculating) {
              calculateSpecificTemplate(tmpl);
            }
          }
        }
      }, { rootMargin: '150px' });

      observer.observe(el);
      cardObservers.set(tmpl.code, observer);
    };
    // 预设模板添加
    const addPreset = (type) => {
      const count = form.boardLayer || 4;
      if (type === '50-single-L1') {
        requirements.value.push({
          id: `req_${Date.now()}`,
          targetZo: 50,
          mode: '单端阻抗（外层）',
          layer: 1,
          upRef: null,
          downRef: 2,
          tolerance: 0.5,
          w1: 8,
          s1: null,
          d1: null
        });
      } else if (type === '90-diff-L1') {
        requirements.value.push({
          id: `req_${Date.now()}`,
          targetZo: 90,
          mode: '差分阻抗（外层）',
          layer: 1,
          upRef: null,
          downRef: 2,
          tolerance: 0.5,
          w1: 4.5,
          s1: 5.5,
          d1: null
        });
      } else if (type === '100-diff-L1') {
        requirements.value.push({
          id: `req_${Date.now()}`,
          targetZo: 100,
          mode: '差分阻抗（外层）',
          layer: 1,
          upRef: null,
          downRef: 2,
          tolerance: 0.5,
          w1: 4,
          s1: 6,
          d1: null
        });
      } else if (type === '50-single-L4') {
        requirements.value.push({
          id: `req_${Date.now()}`,
          targetZo: 50,
          mode: '单端阻抗（外层）',
          layer: count,
          upRef: count - 1,
          downRef: null,
          tolerance: 0.5,
          w1: 8,
          s1: null,
          d1: null
        });
      }
      triggerCalculation();
    };

    const addRequirement = () => {
      requirements.value.push({
        id: `req_${Date.now()}`,
        targetZo: 50,
        mode: '单端阻抗（外层）',
        layer: 1,
        upRef: null,
        downRef: 2,
        tolerance: 0.5,
        w1: 8,
        s1: null,
        d1: null
      });
      triggerCalculation();
    };

    const copyRequirement = (index) => {
      const original = requirements.value[index];
      requirements.value.splice(index + 1, 0, {
        ...JSON.parse(JSON.stringify(original)),
        id: `req_${Date.now()}`
      });
      triggerCalculation();
    };

    const removeRequirement = (index) => {
      if (requirements.value.length <= 1) {
        alert('至少保留一组阻抗需求');
        return;
      }
      requirements.value.splice(index, 1);
      triggerCalculation();
    };

    const onConfigChange = () => {
      loadTemplates();
    };

    const clearCache = () => {
      engine.cache.clear();
      cacheStats.value = engine.cache.getStats();
      alert('本地阻抗缓存已清空');
    };

    const renderSVG = (tmpl) => {
      return StackupVisualizer.renderStackupSVG(tmpl);
    };

    // 导出 CSV
    const exportCSV = () => {
      let csvContent = 'data:text/csv;charset=utf-8,\uFEFF';
      csvContent += '叠构编号,叠构名称,成品厚度(mm),需求阻抗(ohm),阻抗模式,走线层,参考平面,设计线宽(W1),设计线宽(W2),差分线距(S1),共面间距(D1),实际阻抗(ohm),延时(ps/m),介电常数\n';

      for (const tmpl of templates.value) {
        const results = getTemplateResults(tmpl.code);
        for (const res of results) {
          csvContent += `"${tmpl.code}","${tmpl.displayName}","${tmpl.plateThickness}","${res.targetZo}","${res.mode}","L${res.layer}","${res.upRef ? 'L' + res.upRef : '/'}/${res.downRef ? 'L' + res.downRef : '/'}","${res.w1 || ''}","${res.w2 || ''}","${res.s1 || ''}","${res.d1 || ''}","${res.actualZo || ''}","${res.delay || ''}","${res.erEff || ''}"\n`;
        }
      }

      const encodedUri = encodeURI(csvContent);
      const link = document.createElement('a');
      link.setAttribute('href', encodedUri);
      link.setAttribute('download', `PCB阻抗设计报告_${form.boardLayer}层_${form.finishedThickness}mm.csv`);
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    };

    onMounted(() => {
      loadTemplates();
    });

    return {
      form,
      requirements,
      templates,
      filteredTemplates,
      calculating,
      calcStrategy,
      availableLayers,
      cacheStats,
      queueStats,
      getTemplateResults,
      triggerCalculation,
      calculateSpecificTemplate,
      addPreset,
      addRequirement,
      copyRequirement,
      removeRequirement,
      onConfigChange,
      clearCache,
      renderSVG,
      exportCSV,
      pinnedTemplateCode,
      pinTemplate,
      setupCardObserver
    };
  }
});
app.mount('#app');
