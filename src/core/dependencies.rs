#[derive(Debug, Clone)]
pub struct EdgeInfo {
    pub to: usize,
    pub allowed: bool,
    pub label: String,
}

pub type EdgesInfo = Vec<EdgeInfo>;
pub type NodeDependencies = Vec<EdgesInfo>;
