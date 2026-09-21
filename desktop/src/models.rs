// src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardConfig {
    pub plate_type: String,
    pub board_layer: u32,
    pub finished_thickness: f64,
    pub cuprum_thickness: String,
    pub inner_copper_thickness: String,
    pub unit: String,
}

impl Default for BoardConfig {
    fn default() -> Self {
        Self {
            plate_type: "硬板".to_string(),
            board_layer: 4,
            finished_thickness: 1.6,
            cuprum_thickness: "1".to_string(),
            inner_copper_thickness: "0.5".to_string(),
            unit: "mil".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpedanceReq {
    pub id: String,
    pub target_zo: f64,
    pub mode: String,
    pub layer: u32,
    pub up_ref: Option<u32>,
    pub down_ref: Option<u32>,
    pub tolerance: f64,
    pub w1: f64,
    pub s1: Option<f64>,
    pub d1: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicData {
    pub layer_name: Option<String>,
    pub material: String,
    pub material_name: Option<String>,
    pub dielectric_constant: Option<f64>,
    pub dielectric_thick: Option<f64>,
    pub top_conductor_thick: Option<f64>,
    pub material_type: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackupTemplate {
    pub code: String,
    pub display_name: String,
    pub is_common: bool,
    pub plate_thickness: f64,
    pub plate_layer_number: u32,
    pub basic_data_list: Vec<BasicData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpedanceResult {
    pub req_id: String,
    pub target_zo: f64,
    pub mode: String,
    pub layer: u32,
    pub up_ref: Option<u32>,
    pub down_ref: Option<u32>,
    pub tolerance: f64,
    pub w1: Option<f64>,
    pub w2: Option<f64>,
    pub s1: Option<f64>,
    pub d1: Option<f64>,
    pub actual_zo: Option<f64>,
    pub delay: Option<f64>,
    pub er_eff: Option<f64>,
    pub from_cache: bool,
    pub calc_time_ms: u64,
    pub is_loading: bool,
}
