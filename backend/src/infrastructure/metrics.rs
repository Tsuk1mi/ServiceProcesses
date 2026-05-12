use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MetricCount {
    pub name: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HeatmapSnapshot {
    pub hottest_queries: Vec<MetricCount>,
    pub heavy_commands: Vec<MetricCount>,
    pub worker_activity: Vec<MetricCount>,
}

#[derive(Clone, Default)]
pub struct AppMetrics {
    inner: Arc<Mutex<HashMap<String, u64>>>,
}

impl AppMetrics {
    pub fn record_query(&self, name: &str) {
        self.bump(&format!("query:{name}"));
    }

    pub fn record_command(&self, name: &str) {
        self.bump(&format!("command:{name}"));
    }

    pub fn record_worker(&self, name: &str) {
        self.bump(&format!("worker:{name}"));
    }

    fn bump(&self, key: &str) {
        let mut guard = self.inner.lock().expect("metrics mutex poisoned");
        *guard.entry(key.to_string()).or_insert(0) += 1;
    }

    pub fn snapshot(&self) -> HeatmapSnapshot {
        let guard = self.inner.lock().expect("metrics mutex poisoned");
        HeatmapSnapshot {
            hottest_queries: collect(&guard, "query:"),
            heavy_commands: collect(&guard, "command:"),
            worker_activity: collect(&guard, "worker:"),
        }
    }
}

fn collect(map: &HashMap<String, u64>, prefix: &str) -> Vec<MetricCount> {
    let mut items = map
        .iter()
        .filter_map(|(k, v)| {
            k.strip_prefix(prefix).map(|name| MetricCount {
                name: name.to_string(),
                count: *v,
            })
        })
        .collect::<Vec<_>>();
    items.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    items
}
