# PCB 阻抗匹配与高速叠层设计系统 - Web 验证原型 (保留版本)

本项目为 **PCB 阻抗计算系统** 的轻量级 Web 原型参考实现，主要用于嘉立创官方后端求解 API 的协议逆向验证、数据交叉对比以及浏览器环境下的全功能参考实现。

与基于 Rust 的 GPUI 原生桌面应用（`pcb-impedance-gpui`）物理完全解耦，可长期独立运行与维护。

---

## 目录结构

```text
pcb-impedance-web/
├── server.js              # Node.js 原生零依赖反向代理与静态文件托管服务
├── package.json           # 项目配置与 npm scripts
├── start_web.bat          # Windows 一键启动脚本
├── README.md              # 模块说明文档
└── public/
    ├── index.html         # 单页面应用主结构 (Tailwind CSS + 响应式布局)
    ├── app.js             # Vue 3 核心状态管理、多阻抗配置及交互逻辑
    └── js/
        ├── ImpedanceEngine.js    # JLC 官方求解 API 调用引擎
        ├── SmartQueue.js         # 并发限制与自适应请求调度队列
        ├── ImpedanceCache.js     # 本地 LRU 阻抗计算结果缓存
        └── StackupVisualizer.js  # 物理比例 PCB 叠层截面图 SVG 渲染器
```

---

## 核心特性

1. **零第三方 npm 依赖**：
   * `server.js` 使用 Node.js 原生 `http`、`fs`、`path` 模块编写，无需运行 `npm install` 即可直接开箱即用。
2. **反向代理与跨域穿透**：
   * 自动反向代理 `https://tools.jlc.com/api/jlcTools/impedance/` 接口，解决浏览器直接调用时的 CORS 跨域限制与 Referer 校验。
3. **高并发智能队列与本地缓存**：
   * 内置并发请求调度器与 LRU 缓存，在多阻抗全量计算时有效平抑网络尖峰，杜绝前端页面假死。
4. **置顶排序与截面图可视化**：
   * 支持叠构卡片 📌 置顶置前对比，提供物理真实比例 PCB 物理叠层截面图 SVG 动态绘制。

---

## 启动与运行

### 方式 1：双击批处理（推荐）
直接双击运行 `start_web.bat`。

### 方式 2：命令行运行
在当前目录下执行：
```bash
npm start
# 或直接执行:
node server.js
```

服务启动后，在任意现代浏览器中访问：
```text
http://localhost:3000
```
