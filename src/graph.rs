#[derive(Debug)]
pub struct Node {
    // relative_path: String,
    pub id: String, // Unique identifier per node
    pub name: String,
    pub layer: String, // core, io, usecase
    pub node_type: String, // project, namespace, class, folder
    pub color: String,
}

#[derive(Debug)]
pub struct EdgeInfo{
    pub to: usize,
    pub allowed: bool,
    pub label: String,
}
pub type EdgesInfo = Vec<EdgeInfo>;
pub type NodeDependencies = Vec<EdgesInfo>;