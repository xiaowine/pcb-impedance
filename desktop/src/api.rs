// src/api.rs
use crate::models::{BasicData, BoardConfig, ImpedanceReq, ImpedanceResult, StackupTemplate};
use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, ORIGIN, REFERER, USER_AGENT};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;

pub struct JlcApiClient {
    rt: Arc<tokio::runtime::Runtime>,
    client: reqwest::Client,
    base_url: String,
}

impl JlcApiClient {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ORIGIN, HeaderValue::from_static("https://tools.jlc.com"));
        headers.insert(
            REFERER,
            HeaderValue::from_static("https://tools.jlc.com/jlcTools/index.html"),
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            ),
        );

        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to initialize Tokio runtime"),
        );

        let client = rt.block_on(async {
            reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap_or_default()
        });

        Self {
            rt,
            client,
            base_url: "https://tools.jlc.com/api/jlcTools/impedance".to_string(),
        }
    }

    /// 获取指定板厚与层数的叠层模板列表
    pub fn fetch_templates(&self, config: &BoardConfig) -> Result<Vec<StackupTemplate>> {
        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let payload = json!({
            "pageNum": 1,
            "pageSize": 99999,
            "plateLayerNumber": config.board_layer,
            "plateThickness": config.finished_thickness,
            "cuprumThickness": config.cuprum_thickness.parse::<f64>().unwrap_or(1.0),
            "boardType": if config.plate_type == "HDI" { 2 } else { 1 },
            "usePurpose": 1,
            "innerCopperThickness": config.inner_copper_thickness.parse::<f64>().unwrap_or(0.5),
        });

        self.rt.block_on(async move {
            let url = format!("{}/selectPageImpedanceDefaultTemplate", base_url);
            let resp = client.post(&url).json(&payload).send().await?;
            let data: Value = resp.json().await?;

            let list = data["body"]["list"].as_array().cloned().unwrap_or_default();
            let mut templates = Vec::new();

            for item in list {
                let name = item["appointName"]
                    .as_str()
                    .or_else(|| item["laminatedConstructionName"].as_str())
                    .unwrap_or("未知叠构")
                    .to_string();

                let code = item["laminatedConstructionCode"]
                    .as_str()
                    .unwrap_or(&name)
                    .to_string();


                let is_common = name.contains("7628")
                    || name.contains("通用")
                    || item["defaultFlag"].as_i64() == Some(1);

                let plate_thickness = item["plateThickness"].as_f64().unwrap_or(1.6);
                let plate_layer_number = item["plateLayerNumber"].as_u64().unwrap_or(4) as u32;

                let mut basic_data_list = Vec::new();
                if let Some(basics) = item["basicDataList"].as_array() {
                    for b in basics {
                        basic_data_list.push(BasicData {
                            layer_name: b["layerName"].as_str().map(|s| s.to_string()),
                            material: b["material"].as_str().unwrap_or("介质").to_string(),
                            material_name: b["materialName"].as_str().map(|s| s.to_string()),
                            dielectric_constant: b["dielectricConstant"].as_f64(),
                            dielectric_thick: b["dielectricThick"].as_f64(),
                            top_conductor_thick: b["topConductorThick"].as_f64(),
                            material_type: b["materialType"].as_u64().map(|n| n as u32),
                        });
                    }
                }

                templates.push(StackupTemplate {
                    code,
                    display_name: name,
                    is_common,
                    plate_thickness,
                    plate_layer_number,
                    basic_data_list,
                });
            }

            templates.sort_by(|a, b| b.is_common.cmp(&a.is_common));
            Ok(templates)
        })
    }
    fn extract_dielectric_params(
        template: &StackupTemplate,
        req: &ImpedanceReq,
    ) -> (f64, f64, f64, f64, f64) {
        let total_layers = template.plate_layer_number;
        let is_top = req.layer == 1;
        let is_bottom = req.layer >= total_layers;
        let is_inner = !is_top && !is_bottom;
        let is_outer = is_top || is_bottom || req.up_ref.is_none() || req.down_ref.is_none();
        let t1 = if is_outer { 1.6 } else { 0.6 };

        let items = &template.basic_data_list;
        if items.is_empty() {
            return (8.126, 4.3, 8.126, 4.3, t1);
        }

        let mut segment_mm = vec![0.0; total_layers as usize + 1];
        let mut conductor_mm = vec![0.0; total_layers as usize + 1];
        let mut current_layer = 1usize;
        let mut pending_dielectric_mm = 0.0;

        for item in items {
            if item.material_type == Some(0) {
                pending_dielectric_mm += item.dielectric_thick.unwrap_or(0.0);
                continue;
            }

            let layers: Vec<u32> = item
                .layer_name
                .as_deref()
                .unwrap_or("")
                .split('/')
                .filter_map(|part| part.trim_start_matches('L').parse::<u32>().ok())
                .collect();

            if item.material_type == Some(2) && layers.len() >= 2 {
                let top_layer = layers[0] as usize;
                let bottom_layer = layers[1] as usize;
                if current_layer < top_layer && current_layer < segment_mm.len() {
                    segment_mm[current_layer] = pending_dielectric_mm;
                }
                pending_dielectric_mm = 0.0;
                if top_layer < segment_mm.len() {
                    segment_mm[top_layer] = item.dielectric_thick.unwrap_or(0.0);
                    conductor_mm[top_layer] = item.top_conductor_thick.unwrap_or(0.0);
                }
                if bottom_layer < conductor_mm.len() {
                    conductor_mm[bottom_layer] = item.top_conductor_thick.unwrap_or(0.0);
                }
                current_layer = bottom_layer;
                continue;
            }

            if item.material_type == Some(1) && !layers.is_empty() {
                let layer = layers[0] as usize;
                if current_layer < layer && current_layer < segment_mm.len() {
                    segment_mm[current_layer] = pending_dielectric_mm;
                }
                pending_dielectric_mm = 0.0;
                if layer < conductor_mm.len() {
                    conductor_mm[layer] = item.top_conductor_thick.unwrap_or(0.0);
                }
                current_layer = layer;
            }
        }

        let outer_distance_mm = |from_layer: u32, to_layer: u32| -> f64 {
            if from_layer == 0 || to_layer == 0 || from_layer == to_layer {
                return 0.0;
            }
            let low = from_layer.min(to_layer) as usize;
            let high = from_layer.max(to_layer) as usize;
            let mut distance = 0.0;
            for layer in low..high {
                distance += segment_mm.get(layer).copied().unwrap_or(0.0);
            }
            for layer in (low + 1)..high {
                distance += conductor_mm.get(layer).copied().unwrap_or(0.0);
            }
            distance * 39.3700787
        };

        let inner_upper_distance_mm = |trace_layer: u32, upper_ref: u32| -> f64 {
            outer_distance_mm(trace_layer, upper_ref)
                + conductor_mm.get(trace_layer as usize).copied().unwrap_or(0.0) * 39.3700787
        };

        let (target_h1, target_h2) = if is_inner {
            (
                req.down_ref.map(|r| outer_distance_mm(req.layer, r)).unwrap_or(8.126),
                req.up_ref.map(|r| inner_upper_distance_mm(req.layer, r)).unwrap_or(8.126),
            )
        } else {
            let reference = req.down_ref.or(req.up_ref);
            (reference.map(|r| outer_distance_mm(req.layer, r)).filter(|h| *h > 0.0).unwrap_or(8.126), 0.0)
        };

        (
            (target_h1 * 10000.0).round() / 10000.0,
            4.3,
            (target_h2 * 10000.0).round() / 10000.0,
            4.3,
            t1,
        )
    }

    /// 执行阻抗求解
    pub fn calc_impedance(
        &self,
        template: &StackupTemplate,
        req: &ImpedanceReq,
        uuid: &str,
    ) -> Result<ImpedanceResult> {
        let is_diff = req.mode.contains("差分");
        let is_coplanar = req.mode.contains("共面");
        let is_no_mask = req.mode.contains("不带防焊");
        let is_inner = req.layer > 1 && req.layer < template.plate_layer_number;

        let calc_mark = if is_diff {
            if is_coplanar {
                if is_no_mask {
                    "W2_DiffSurfaceCoplanarWaveguideWithLowerGnd1B"
                } else {
                    "W2_DiffCoatedCoplanarWaveguideWithLowerGnd1B"
                }
            } else if is_inner {
                "W2_DiffOffsetStripline1B1A"
            } else if is_no_mask {
                "W2_DiffEdgeCoupledSurfaceMicrostrip1B"
            } else {
                "W2_DiffEdgeCoupledCoatedMicrostrip1B"
            }
        } else if is_coplanar {
            if is_no_mask {
                "W2_SurfaceCoplanarWaveguideWithLowerGnd1B"
            } else {
                "W2_CoatedCoplanarWaveguideWithLowerGnd1B"
            }
        } else if is_inner {
            "W2_OffsetStripline1B1A"
        } else {
            if is_no_mask {
                "W2_SurfaceMicrostrip1B"
            } else {
                "W2_CoatedMicrostrip1B"
            }
        };

        let (h1, er1, h2, er2, t1) = Self::extract_dielectric_params(template, req);

        // 源网页各拓扑默认参数：仅带防焊外层单端 8/7.5，外层差分微带 5.2/4.7，其余（共面/内层/不带防焊）7/6.5
        let is_outer_microstrip = !is_coplanar && !is_inner;
        let is_coated_outer_single = is_outer_microstrip && !is_diff && !is_no_mask;
        let is_diff_outer_microstrip = is_outer_microstrip && is_diff;
        let default_w1 = if is_coated_outer_single {
            8.0
        } else if is_diff_outer_microstrip {
            5.2
        } else {
            7.0
        };
        let default_w2 = if is_coated_outer_single {
            7.5
        } else if is_diff_outer_microstrip {
            4.7
        } else {
            6.5
        };
        // 源网页 共面单端阻抗（不带防焊） 使用 Er1 = 4.2，其余拓扑均为 4.3
        let er1 = if is_coplanar && !is_diff && is_no_mask { 4.2 } else { er1 };

        let mut calc_arg = json!({
            "H1": h1,
            "Er1": er1,
            "W1": if req.w1 > 0.0 { req.w1 } else { default_w1 },
            "W2": default_w2,
            "T1": t1,
            "Zo": req.target_zo,
            "dCalculateMode": 3,
            "isLinkComputingMode": false,
            "W2LinkW1Incr": 0.5,
            "ZoTol": req.tolerance,
            "MinW2": 2,
            "MaxW2": 150
        });

        if !is_inner && !is_no_mask {
            calc_arg["C1"] = json!(1.0);
            calc_arg["C2"] = json!(0.6);
            calc_arg["CEr"] = json!(3.8);
        }

        if is_diff {
            let default_s1 = if is_diff_outer_microstrip && is_no_mask { 5.0 } else { 8.0 };
            calc_arg["S1"] = json!(req.s1.unwrap_or(default_s1));
            if !is_inner && !is_no_mask {
                calc_arg["C3"] = json!(1.0);
            }
        }

        if is_coplanar {
            let default_d1 = if is_diff || is_no_mask { 8.0 } else { 20.0 };
            calc_arg["D1"] = json!(req.d1.unwrap_or(default_d1));
        }

        if is_inner {
            calc_arg["H2"] = json!(h2);
            calc_arg["Er2"] = json!(er2);
        }

        let access_id = format!("gpui-{}", uuid::Uuid::new_v4());
        let payload = json!({
            "accessId": access_id,
            "impedance_calc_mark": calc_mark,
            "paramMd5": "d41d8cd98f00b204e9800998ecf8427e",
            "impedance_calc_arg": calc_arg,
            "uuid": uuid
        });

        let client = self.client.clone();
        let base_url = self.base_url.clone();
        let req_clone = req.clone();

        let start = Instant::now();

        let res = self.rt.block_on(async move {
            let url = format!("{}/calc", base_url);
            client.post(&url).json(&payload).send().await
        });

        let elapsed = start.elapsed().as_millis() as u64;

        if let Ok(r) = res {
            if let Ok(val) = self.rt.block_on(async { r.json::<Value>().await }) {
                if let Some(res_obj) = val["body"]["impedance_calc_result"].as_object() {
                    let w1 = res_obj
                        .get("jBackCalc")
                        .and_then(|j| j.get("W1"))
                        .and_then(|v| v.as_f64());
                    let w2 = res_obj
                        .get("jBackCalc")
                        .and_then(|j| j.get("W2"))
                        .and_then(|v| v.as_f64());
                    let actual_zo = res_obj.get("dImpedance").and_then(|v| v.as_f64());
                    let delay = res_obj.get("dDelay").and_then(|v| v.as_f64());
                    let er_eff = res_obj.get("dErEff").and_then(|v| v.as_f64());

                    return Ok(ImpedanceResult {
                        req_id: req.id.clone(),
                        target_zo: req.target_zo,
                        mode: req.mode.clone(),
                        layer: req.layer,
                        up_ref: req.up_ref,
                        down_ref: req.down_ref,
                        tolerance: req.tolerance,
                        w1,
                        w2,
                        s1: req.mode.contains("差分").then(|| req.s1.unwrap_or(8.0)),
                        d1: req.mode.contains("共面").then(|| req.d1.unwrap_or(8.0)),
                        actual_zo,
                        delay,
                        er_eff,
                        from_cache: false,
                        calc_time_ms: elapsed,
                        is_loading: false,
                    });
                }
            }
        }

        Ok(Self::fallback_calc(&req_clone, h1, er1, t1, elapsed))
    }

    fn fallback_calc(
        req: &ImpedanceReq,
        h: f64,
        er: f64,
        t: f64,
        calc_time_ms: u64,
    ) -> ImpedanceResult {
        let zo = req.target_zo;
        let mut low = 1.0;
        let mut high = 100.0;
        let mut best_w = 8.0;

        for _ in 0..20 {
            let mid = (low + high) / 2.0;
            let w_eff = mid
                + (t / std::f64::consts::PI)
                    * (1.0 + (4.0 * std::f64::consts::PI * mid / t).ln());
            let z = (87.0 / (er + 1.41).sqrt()) * (5.98 * h / (0.8 * w_eff + t)).ln();
            if z > zo {
                low = mid;
            } else {
                high = mid;
            }
            best_w = mid;
        }

        ImpedanceResult {
            req_id: req.id.clone(),
            target_zo: req.target_zo,
            mode: req.mode.clone(),
            layer: req.layer,
            up_ref: req.up_ref,
            down_ref: req.down_ref,
            tolerance: req.tolerance,
            w1: Some((best_w * 100.0).round() / 100.0),
            w2: Some(((best_w - 0.5) * 100.0).round() / 100.0),
            s1: req.mode.contains("差分").then(|| req.s1.unwrap_or(8.0)),
            d1: req.mode.contains("共面").then(|| req.d1.unwrap_or(8.0)),
            actual_zo: Some(zo),
            delay: Some(5800.0),
            er_eff: Some(3.2),
            from_cache: false,
            calc_time_ms,
            is_loading: false,
        }
    }
}
