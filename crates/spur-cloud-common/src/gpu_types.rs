use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuPool {
    pub gpu_type: String,
    pub total: u32,
    pub available: u32,
    pub allocated: u32,
    pub memory_mb: u64,
    pub nodes: Vec<GpuNodeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuNodeInfo {
    pub name: String,
    pub total_gpus: u32,
    pub available_gpus: u32,
    pub state: String,
}
