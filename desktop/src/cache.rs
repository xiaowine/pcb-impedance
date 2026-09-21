// src/cache.rs
use crate::models::{ImpedanceReq, ImpedanceResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ImpedanceCache {
    store: Arc<Mutex<HashMap<String, ImpedanceResult>>>,
    hits: Arc<Mutex<usize>>,
    misses: Arc<Mutex<usize>>,
}

impl ImpedanceCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    pub fn generate_key(template_code: &str, req: &ImpedanceReq) -> String {
        format!(
            "{}_{}_{:.3}_{}_{}_{}_{:?}_{:?}_{:.3}_{:?}_{:?}",
            template_code,
            req.id,
            req.target_zo,
            req.mode,
            req.layer,
            req.tolerance,
            req.up_ref,
            req.down_ref,
            req.w1,
            req.s1,
            req.d1,
        )
    }

    pub fn get(&self, key: &str) -> Option<ImpedanceResult> {
        let store = self.store.lock().ok()?;
        if let Some(res) = store.get(key).cloned() {
            if let Ok(mut h) = self.hits.lock() {
                *h += 1;
            }
            Some(res)
        } else {
            if let Ok(mut m) = self.misses.lock() {
                *m += 1;
            }
            None
        }
    }

    pub fn set(&self, key: String, mut result: ImpedanceResult) {
        result.from_cache = true;
        if let Ok(mut store) = self.store.lock() {
            store.insert(key, result);
        }
    }

    pub fn stats(&self) -> (usize, usize, String) {
        let h = self.hits.lock().map(|v| *v).unwrap_or(0);
        let m = self.misses.lock().map(|v| *v).unwrap_or(0);
        let total = h + m;
        let rate = if total > 0 {
            format!("{:.1}%", (h as f64 / total as f64) * 100.0)
        } else {
            "0.0%".to_string()
        };
        (h, m, rate)
    }
}
