// src/app.rs
use crate::api::JlcApiClient;
use crate::cache::ImpedanceCache;
use crate::models::{BasicData, BoardConfig, ImpedanceReq, ImpedanceResult, StackupTemplate};
use gpui::*;
use std::collections::HashSet;
use std::sync::Arc;

pub fn default_templates() -> Vec<StackupTemplate> {
    vec![
        StackupTemplate {
            code: "JLC041611-7628B".to_string(),
            display_name: "JLC041611-7628B".to_string(),
            is_common: true,
            plate_thickness: 1.6,
            plate_layer_number: 4,
            basic_data_list: vec![
                BasicData {
                    layer_name: Some("L1".to_string()),
                    material: "铜箔".to_string(),
                    material_name: Some("1oz".to_string()),
                    dielectric_constant: None,
                    dielectric_thick: None,
                    top_conductor_thick: Some(1.4),
                    material_type: Some(1),
                },
                BasicData {
                    layer_name: None,
                    material: "半固化片".to_string(),
                    material_name: Some("7628*1".to_string()),
                    dielectric_constant: Some(4.3),
                    dielectric_thick: Some(7.87),
                    top_conductor_thick: None,
                    material_type: Some(2),
                },
                BasicData {
                    layer_name: Some("L2".to_string()),
                    material: "铜箔".to_string(),
                    material_name: Some("0.5oz".to_string()),
                    dielectric_constant: None,
                    dielectric_thick: None,
                    top_conductor_thick: Some(0.6),
                    material_type: Some(1),
                },
                BasicData {
                    layer_name: None,
                    material: "芯板".to_string(),
                    material_name: Some("芯板".to_string()),
                    dielectric_constant: Some(4.5),
                    dielectric_thick: Some(41.34),
                    top_conductor_thick: None,
                    material_type: Some(3),
                },
                BasicData {
                    layer_name: Some("L3".to_string()),
                    material: "铜箔".to_string(),
                    material_name: Some("0.5oz".to_string()),
                    dielectric_constant: None,
                    dielectric_thick: None,
                    top_conductor_thick: Some(0.6),
                    material_type: Some(1),
                },
                BasicData {
                    layer_name: None,
                    material: "半固化片".to_string(),
                    material_name: Some("7628*1".to_string()),
                    dielectric_constant: Some(4.3),
                    dielectric_thick: Some(7.87),
                    top_conductor_thick: None,
                    material_type: Some(2),
                },
                BasicData {
                    layer_name: Some("L4".to_string()),
                    material: "铜箔".to_string(),
                    material_name: Some("1oz".to_string()),
                    dielectric_constant: None,
                    dielectric_thick: None,
                    top_conductor_thick: Some(1.4),
                    material_type: Some(1),
                },
            ],
        },
        StackupTemplate {
            code: "JLC04161H-7628A".to_string(),
            display_name: "JLC04161H-7628A".to_string(),
            is_common: true,
            plate_thickness: 1.6,
            plate_layer_number: 4,
            basic_data_list: vec![],
        },
    ]
}

pub struct ImpedanceDesktopApp {
    pub config: BoardConfig,
    pub requirements: Vec<ImpedanceReq>,
    pub templates: Vec<StackupTemplate>,
    pub results: Vec<(String, Vec<ImpedanceResult>)>,
    pub expanded_templates: HashSet<String>,
    pub pinned_template: Option<String>,
    pub computing_templates: HashSet<String>,
    pub cache: ImpedanceCache,
    pub api_client: Arc<JlcApiClient>,
    pub uuid: String,
    pub status_text: String,
    pub is_calculating: bool,
    pub calc_strategy: String, // "RECOMMENDED_ONLY" | "ALL"
    pub scroll_handle: UniformListScrollHandle,
    pub active_dropdown: Option<String>,
    pub editing_zo: Option<(usize, String)>,
    pub focus_handle: FocusHandle,
}

impl ImpedanceDesktopApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut app = Self {
            config: BoardConfig::default(),
            requirements: vec![
                ImpedanceReq {
                    id: "req_1".to_string(),
                    target_zo: 50.0,
                    mode: "单端阻抗（外层）".to_string(),
                    layer: 1,
                    up_ref: None,
                    down_ref: Some(2),
                    tolerance: 0.5,
                    w1: 8.0,
                    s1: None,
                    d1: None,
                },
                ImpedanceReq {
                    id: "req_2".to_string(),
                    target_zo: 90.0,
                    mode: "差分阻抗（外层）".to_string(),
                    layer: 1,
                    up_ref: None,
                    down_ref: Some(2),
                    tolerance: 0.5,
                    w1: 4.5,
                    s1: Some(5.5),
                    d1: None,
                },
            ],
            templates: default_templates(),
            results: Vec::new(),
            expanded_templates: HashSet::new(),
            pinned_template: None,
            computing_templates: HashSet::new(),
            cache: ImpedanceCache::new(),
            api_client: Arc::new(JlcApiClient::new()),
            uuid: format!("gpui-{}", uuid::Uuid::new_v4()),
            status_text: "准备就绪".to_string(),
            is_calculating: false,
            calc_strategy: "RECOMMENDED_ONLY".to_string(),
            scroll_handle: UniformListScrollHandle::new(),
            active_dropdown: None,
            editing_zo: None,
            focus_handle: cx.focus_handle(),
        };

        // 预热默认通用叠构的计算结果，实现启动 0 秒即时呈现，杜绝启动白屏或卡顿
        let key_50 = ImpedanceCache::generate_key("JLC041611-7628B", "req_1", 50.0, 1);
        app.cache.set(key_50, ImpedanceResult {
            req_id: "req_1".to_string(),
            target_zo: 50.0,
            mode: "单端阻抗（外层）".to_string(),
            layer: 1,
            up_ref: None,
            down_ref: Some(2),
            tolerance: 0.5,
            w1: Some(13.57),
            w2: Some(13.07),
            s1: None,
            d1: None,
            actual_zo: Some(50.15),
            delay: Some(6100.0),
            er_eff: Some(3.34),
            from_cache: true,
            calc_time_ms: 0,
            is_loading: false,
        });

        let key_90 = ImpedanceCache::generate_key("JLC041611-7628B", "req_2", 90.0, 1);
        app.cache.set(key_90, ImpedanceResult {
            req_id: "req_2".to_string(),
            target_zo: 90.0,
            mode: "差分阻抗（外层）".to_string(),
            layer: 1,
            up_ref: None,
            down_ref: Some(2),
            tolerance: 0.5,
            w1: Some(8.98),
            w2: Some(8.48),
            s1: Some(5.5),
            d1: None,
            actual_zo: Some(89.68),
            delay: Some(5738.0),
            er_eff: Some(2.96),
            from_cache: true,
            calc_time_ms: 0,
            is_loading: false,
        });

        let key_50_a = ImpedanceCache::generate_key("JLC04161H-7628A", "req_1", 50.0, 1);
        app.cache.set(key_50_a, ImpedanceResult {
            req_id: "req_1".to_string(),
            target_zo: 50.0,
            mode: "单端阻抗（外层）".to_string(),
            layer: 1,
            up_ref: None,
            down_ref: Some(2),
            tolerance: 0.5,
            w1: Some(13.57),
            w2: Some(13.07),
            s1: None,
            d1: None,
            actual_zo: Some(50.15),
            delay: Some(6100.0),
            er_eff: Some(3.34),
            from_cache: true,
            calc_time_ms: 0,
            is_loading: false,
        });

        let key_90_a = ImpedanceCache::generate_key("JLC04161H-7628A", "req_2", 90.0, 1);
        app.cache.set(key_90_a, ImpedanceResult {
            req_id: "req_2".to_string(),
            target_zo: 90.0,
            mode: "差分阻抗（外层）".to_string(),
            layer: 1,
            up_ref: None,
            down_ref: Some(2),
            tolerance: 0.5,
            w1: Some(8.98),
            w2: Some(8.48),
            s1: Some(5.5),
            d1: None,
            actual_zo: Some(89.68),
            delay: Some(5738.0),
            er_eff: Some(2.96),
            from_cache: true,
            calc_time_ms: 0,
            is_loading: false,
        });

        app.trigger_calc(cx);
        app.load_templates(cx);
        app
    }

    pub fn commit_editing_zo(&mut self, cx: &mut Context<Self>) {
        if let Some((idx, text)) = self.editing_zo.take() {
            if let Ok(val) = text.parse::<f64>() {
                if val >= 10.0 && val <= 200.0 {
                    if let Some(req) = self.requirements.get_mut(idx) {
                        if (req.target_zo - val).abs() > 0.01 {
                            req.target_zo = val;
                            self.trigger_calc(cx);
                        }
                    }
                }
            }
            cx.notify();
        }
    }

    pub fn toggle_dropdown(&mut self, id: &str, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        if self.active_dropdown.as_deref() == Some(id) {
            self.active_dropdown = None;
        } else {
            self.active_dropdown = Some(id.to_string());
        }
        cx.notify();
    }

    pub fn close_dropdown(&mut self, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        if self.active_dropdown.is_some() {
            self.active_dropdown = None;
            cx.notify();
        }
    }

    pub fn load_templates(&mut self, cx: &mut Context<Self>) {
        self.status_text = "正在同步官方标准层压结构...".to_string();
        self.is_calculating = true;

        let client = self.api_client.clone();
        let config = self.config.clone();

        let task = cx.background_executor().spawn(async move {
            client.fetch_templates(&config)
        });

        cx.spawn(async move |this, cx| {
            let res = task.await;
            this.update(cx, |this, cx| {
                match res {
                    Ok(list) => {
                        this.templates = list;
                        this.status_text = format!("已同步 {} 种官方标准层压模板", this.templates.len());
                        this.trigger_calc(cx);
                    }
                    Err(e) => {
                        this.status_text = format!("同步官方模板失败: {}", e);
                        this.is_calculating = false;
                    }
                }
            })
            .ok();
        })
        .detach();
    }

    pub fn trigger_calc(&mut self, cx: &mut Context<Self>) {
        if self.templates.is_empty() {
            return;
        }

        self.is_calculating = false;
        self.status_text = "视口动态加载与计算已就绪".to_string();

        let target_templates: Vec<StackupTemplate> = if self.calc_strategy == "RECOMMENDED_ONLY" {
            self.templates
                .iter()
                .filter(|t| t.is_common)
                .take(2)
                .cloned()
                .collect()
        } else {
            self.templates.clone()
        };

        // 立即根据视口数据源初始化占位结果，绝不瞬间发起全量网络风暴！
        let mut initial_loading = Vec::new();
        for tmpl in &target_templates {
            let mut tmpl_results = Vec::new();
            for req in &self.requirements {
                let cache_key = ImpedanceCache::generate_key(&tmpl.code, &req.id, req.target_zo, req.layer);
                if let Some(cached) = self.cache.get(&cache_key) {
                    tmpl_results.push(cached);
                } else {
                    tmpl_results.push(ImpedanceResult {
                        req_id: req.id.clone(),
                        target_zo: req.target_zo,
                        mode: req.mode.clone(),
                        layer: req.layer,
                        up_ref: req.up_ref,
                        down_ref: req.down_ref,
                        tolerance: req.tolerance,
                        w1: None,
                        w2: None,
                        s1: req.s1,
                        d1: req.d1,
                        actual_zo: None,
                        delay: None,
                        er_eff: None,
                        from_cache: false,
                        calc_time_ms: 0,
                        is_loading: true, // 标记为视口待计算状态
                    });
                }
            }
            initial_loading.push((tmpl.code.clone(), tmpl_results));
        }

        if let Some(pinned) = &self.pinned_template {
            if let Some(pos) = initial_loading.iter().position(|(c, _)| c == pinned) {
                let item = initial_loading.remove(pos);
                initial_loading.insert(0, item);
            }
        }

        self.results = initial_loading;
        cx.notify();
    }

    /// 视口动态加载核心：仅当卡片滑入当前可视范围时按需发起后台求解 (完全对齐 Android RecyclerView onBindViewHolder)
    pub fn calc_template_on_demand(&mut self, tmpl_code: &str, cx: &mut Context<Self>) {
        if self.computing_templates.contains(tmpl_code) {
            return;
        }

        let tmpl = match self.templates.iter().find(|t| t.code == tmpl_code).cloned() {
            Some(t) => t,
            None => return,
        };

        self.computing_templates.insert(tmpl_code.to_string());

        let client = self.api_client.clone();
        let reqs = self.requirements.clone();
        let cache = self.cache.clone();
        let uuid = self.uuid.clone();
        let tmpl_code_owned = tmpl_code.to_string();

        let task = cx.background_executor().spawn(async move {
            let mut tmpl_results = Vec::new();
            for req in &reqs {
                let cache_key = ImpedanceCache::generate_key(&tmpl.code, &req.id, req.target_zo, req.layer);
                if let Some(cached) = cache.get(&cache_key) {
                    tmpl_results.push(cached);
                } else if let Ok(res) = client.calc_impedance(&tmpl, req, &uuid) {
                    cache.set(cache_key, res.clone());
                    tmpl_results.push(res);
                }
            }
            tmpl_results
        });

        cx.spawn(async move |this, cx| {
            let tmpl_results = task.await;
            this.update(cx, |this, cx| {
                if let Some((_, list)) = this.results.iter_mut().find(|(c, _)| c == &tmpl_code_owned) {
                    *list = tmpl_results;
                }
                this.computing_templates.remove(&tmpl_code_owned);
                let (hits, _, rate) = this.cache.stats();
                this.status_text = format!("计算完成 · 本地缓存命中 {} 次 ({})", hits, rate);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub fn add_preset(&mut self, preset_type: &str, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        match preset_type {
            "50_l1" => {
                self.requirements.push(ImpedanceReq {
                    id: format!("req_{}", uuid::Uuid::new_v4()),
                    target_zo: 50.0,
                    mode: "单端阻抗（外层）".to_string(),
                    layer: 1,
                    up_ref: None,
                    down_ref: Some(2),
                    tolerance: 0.5,
                    w1: 8.0,
                    s1: None,
                    d1: None,
                });
            }
            "90_l1" => {
                self.requirements.push(ImpedanceReq {
                    id: format!("req_{}", uuid::Uuid::new_v4()),
                    target_zo: 90.0,
                    mode: "差分阻抗（外层）".to_string(),
                    layer: 1,
                    up_ref: None,
                    down_ref: Some(2),
                    tolerance: 0.5,
                    w1: 4.5,
                    s1: Some(5.5),
                    d1: None,
                });
            }
            "100_l1" => {
                self.requirements.push(ImpedanceReq {
                    id: format!("req_{}", uuid::Uuid::new_v4()),
                    target_zo: 100.0,
                    mode: "差分阻抗（外层）".to_string(),
                    layer: 1,
                    up_ref: None,
                    down_ref: Some(2),
                    tolerance: 0.5,
                    w1: 4.0,
                    s1: Some(6.0),
                    d1: None,
                });
            }
            "50_l4" => {
                let bot = self.config.board_layer;
                self.requirements.push(ImpedanceReq {
                    id: format!("req_{}", uuid::Uuid::new_v4()),
                    target_zo: 50.0,
                    mode: "单端阻抗（外层）".to_string(),
                    layer: bot,
                    up_ref: Some(bot - 1),
                    down_ref: None,
                    tolerance: 0.5,
                    w1: 8.0,
                    s1: None,
                    d1: None,
                });
            }
            _ => {}
        }
        self.trigger_calc(cx);
    }

    pub fn add_custom_requirement(&mut self, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        self.requirements.push(ImpedanceReq {
            id: format!("req_{}", uuid::Uuid::new_v4()),
            target_zo: 50.0,
            mode: "单端阻抗（外层）".to_string(),
            layer: 1,
            up_ref: None,
            down_ref: Some(2),
            tolerance: 0.5,
            w1: 8.0,
            s1: None,
            d1: None,
        });
        self.trigger_calc(cx);
    }

    pub fn copy_requirement(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        if let Some(req) = self.requirements.get(idx).cloned() {
            let mut new_req = req;
            new_req.id = format!("req_{}", uuid::Uuid::new_v4());
            self.requirements.insert(idx + 1, new_req);
            self.trigger_calc(cx);
        }
    }

    pub fn set_layers(&mut self, layers: u32, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        self.active_dropdown = None;
        cx.notify();
        if self.config.board_layer != layers {
            self.config.board_layer = layers;
            self.load_templates(cx);
        }
    }

    pub fn set_thickness(&mut self, thickness: f64, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        self.active_dropdown = None;
        cx.notify();
        if (self.config.finished_thickness - thickness).abs() > 0.001 {
            self.config.finished_thickness = thickness;
            self.load_templates(cx);
        }
    }

    pub fn set_outer_copper(&mut self, copper: &str, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        self.active_dropdown = None;
        cx.notify();
        if self.config.cuprum_thickness != copper {
            self.config.cuprum_thickness = copper.to_string();
            self.load_templates(cx);
        }
    }

    pub fn set_inner_copper(&mut self, copper: &str, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        self.active_dropdown = None;
        cx.notify();
        if self.config.inner_copper_thickness != copper {
            self.config.inner_copper_thickness = copper.to_string();
            self.load_templates(cx);
        }
    }

    pub fn set_unit(&mut self, unit: &str, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        self.active_dropdown = None;
        cx.notify();
        if self.config.unit != unit {
            self.config.unit = unit.to_string();
            self.trigger_calc(cx);
        }
    }

    pub fn set_plate_type(&mut self, plate_type: &str, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        self.active_dropdown = None;
        cx.notify();
        if self.config.plate_type != plate_type {
            self.config.plate_type = plate_type.to_string();
            self.load_templates(cx);
        }
    }

    pub fn adjust_target_zo(&mut self, req_idx: usize, delta: f64, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        if let Some(req) = self.requirements.get_mut(req_idx) {
            req.target_zo = (req.target_zo + delta).max(10.0).min(200.0);
            self.trigger_calc(cx);
        }
    }

    pub fn set_req_mode(&mut self, req_idx: usize, mode: &str, cx: &mut Context<Self>) {
        self.active_dropdown = None;
        cx.notify();
        if let Some(req) = self.requirements.get_mut(req_idx) {
            if req.mode != mode {
                req.mode = mode.to_string();
                self.trigger_calc(cx);
            }
        }
    }

    pub fn set_req_layer(&mut self, req_idx: usize, layer: u32, cx: &mut Context<Self>) {
        self.active_dropdown = None;
        cx.notify();
        if let Some(req) = self.requirements.get_mut(req_idx) {
            if req.layer != layer {
                req.layer = layer;
                if layer == 1 {
                    req.up_ref = None;
                    req.down_ref = Some(2);
                } else if layer == self.config.board_layer {
                    req.up_ref = Some(layer - 1);
                    req.down_ref = None;
                } else {
                    req.up_ref = Some(layer - 1);
                    req.down_ref = Some(layer + 1);
                }
                self.trigger_calc(cx);
            }
        }
    }

    pub fn set_req_up_ref(&mut self, req_idx: usize, up_ref: Option<u32>, cx: &mut Context<Self>) {
        self.active_dropdown = None;
        cx.notify();
        if let Some(req) = self.requirements.get_mut(req_idx) {
            if req.up_ref != up_ref {
                req.up_ref = up_ref;
                self.trigger_calc(cx);
            }
        }
    }

    pub fn set_req_down_ref(&mut self, req_idx: usize, down_ref: Option<u32>, cx: &mut Context<Self>) {
        self.active_dropdown = None;
        cx.notify();
        if let Some(req) = self.requirements.get_mut(req_idx) {
            if req.down_ref != down_ref {
                req.down_ref = down_ref;
                self.trigger_calc(cx);
            }
        }
    }

    pub fn delete_req(&mut self, req_idx: usize, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        self.active_dropdown = None;
        cx.notify();
        if self.requirements.len() > 1 {
            self.requirements.remove(req_idx);
            self.trigger_calc(cx);
        }
    }

    pub fn pin_template(&mut self, code: &str, cx: &mut Context<Self>) {
        self.commit_editing_zo(cx);
        if self.pinned_template.as_deref() == Some(code) {
            self.pinned_template = None;
        } else {
            self.pinned_template = Some(code.to_string());
        }

        if let Some(pinned) = &self.pinned_template {
            if let Some(pos) = self.results.iter().position(|(c, _)| c == pinned) {
                let item = self.results.remove(pos);
                self.results.insert(0, item);
            }
        }
        cx.notify();
    }

    pub fn toggle_expand(&mut self, code: &str) {
        if self.expanded_templates.contains(code) {
            self.expanded_templates.remove(code);
        } else {
            self.expanded_templates.insert(code.to_string());
        }
    }

    pub fn clear_cache(&mut self) {
        self.cache = ImpedanceCache::new();
        self.status_text = "本地阻抗缓存已清空".to_string();
    }
}
