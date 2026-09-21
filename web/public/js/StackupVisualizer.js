// public/js/StackupVisualizer.js
/**
 * 现代 PCB 叠构剖面图 SVG 渲染器
 * 真实还原铜箔、PP、Core 芯板、绿油阻焊及走线微带线几何截面
 */
export class StackupVisualizer {
  /**
   * 渲染叠层截面 SVG
   * @param {Object} template 叠层模板对象，含 basicDataList
   * @param {Object} options 渲染配置 { width, height, activeLayer }
   */
  static renderStackupSVG(template, options = {}) {
    const width = options.width || 480;
    const list = template.basicDataList || [];
    if (!list.length) {
      return `<svg viewBox="0 0 480 100" width="100%" height="100" preserveAspectRatio="xMinYMid meet" class="max-w-full"><text x="20" y="50" fill="#94a3b8">暂无叠层详细材料数据</text></svg>`;
    }

    const marginX = 80;
    const contentWidth = width - marginX - 100;
    let currentY = 20;

    const layerItems = [];
    let totalThick = 0;

    for (const item of list) {
      const thick = Number(item.dielectricThick || item.thickness || 0.1);
      totalThick += thick;
      layerItems.push({
        layerName: item.layerName || '',
        material: item.material || '介质',
        materialName: item.materialName || '',
        thick,
        er: item.dielectricConstant || 4.3,
        type: item.materialType // 1: 铜箔, 2: PP 半固化片, 3: Core 芯板
      });
    }

    // 计算各层视觉高度（按比例映射，最小 14px，最大 45px）
    const visualLayers = layerItems.map(item => {
      const height = Math.max(16, Math.min(48, Math.round((item.thick / (totalThick || 1)) * 220)));
      return { ...item, height };
    });

    const totalHeight = visualLayers.reduce((sum, l) => sum + l.height, 0) + 40;

    let svgContent = `<svg viewBox="0 0 ${width} ${totalHeight}" width="100%" height="${totalHeight}" xmlns="http://www.w3.org/2000/svg" class="select-none font-sans">
      <defs>
        <!-- 铜箔渐变 -->
        <linearGradient id="copperGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#f59e0b" />
          <stop offset="100%" stop-color="#d97706" />
        </linearGradient>
        <!-- PP 半固化片渐变 -->
        <linearGradient id="ppGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#06b6d4" stop-opacity="0.25" />
          <stop offset="100%" stop-color="#0891b2" stop-opacity="0.35" />
        </linearGradient>
        <!-- 芯板 Core 渐变 -->
        <linearGradient id="coreGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#3b82f6" stop-opacity="0.25" />
          <stop offset="100%" stop-color="#2563eb" stop-opacity="0.4" />
        </linearGradient>
        <!-- 绿油阻焊 -->
        <linearGradient id="maskGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="#10b981" stop-opacity="0.6" />
          <stop offset="100%" stop-color="#059669" stop-opacity="0.8" />
        </linearGradient>
      </defs>
    `;

    // 顶层阻焊提示
    svgContent += `
      <rect x="${marginX}" y="${currentY}" width="${contentWidth}" height="8" fill="url(#maskGrad)" rx="2" />
      <text x="${marginX + 8}" y="${currentY + 6}" font-size="9" fill="#047857" font-weight="600">Solder Mask (阻焊层 Er=3.8)</text>
    `;
    currentY += 10;

    // 循环绘制各物理层
    for (let i = 0; i < visualLayers.length; i++) {
      const layer = visualLayers[i];
      let fill = 'url(#ppGrad)';
      let stroke = '#0891b2';
      let textColor = '#0e7490';

      if (layer.type === 1 || layer.material.includes('铜')) {
        fill = 'url(#copperGrad)';
        stroke = '#b45309';
        textColor = '#ffffff';
      } else if (layer.type === 3 || layer.material.includes('芯板')) {
        fill = 'url(#coreGrad)';
        stroke = '#2563eb';
        textColor = '#1d4ed8';
      }

      // 层名标签 (左侧)
      if (layer.layerName) {
        svgContent += `
          <text x="24" y="${currentY + layer.height / 2 + 4}" font-size="12" font-weight="bold" fill="#334155">${layer.layerName}</text>
          <line x1="50" y1="${currentY + layer.height / 2}" x2="${marginX - 5}" y2="${currentY + layer.height / 2}" stroke="#cbd5e1" stroke-dasharray="3,3" />
        `;
      }

      // 介质/铜层矩形
      svgContent += `
        <rect x="${marginX}" y="${currentY}" width="${contentWidth}" height="${layer.height}" fill="${fill}" stroke="${stroke}" stroke-width="1" rx="2" />
        <text x="${marginX + 12}" y="${currentY + layer.height / 2 + 4}" font-size="11" font-weight="500" fill="${textColor}">
          ${layer.materialName || layer.material} ${layer.er ? `(Er=${layer.er})` : ''}
        </text>
        <text x="${marginX + contentWidth - 8}" y="${currentY + layer.height / 2 + 4}" font-size="11" text-anchor="end" fill="#64748b" font-family="monospace">
          ${layer.thick.toFixed(4)} mm
        </text>
      `;

      currentY += layer.height + 2;
    }

    // 底层阻焊提示
    svgContent += `
      <rect x="${marginX}" y="${currentY}" width="${contentWidth}" height="8" fill="url(#maskGrad)" rx="2" />
      <text x="${marginX + 8}" y="${currentY + 6}" font-size="9" fill="#047857" font-weight="600">Solder Mask (阻焊层 Er=3.8)</text>
    `;

    svgContent += `</svg>`;
    return svgContent;
  }
}
