
#[derive(Default, Clone)]
pub struct Message {
    pub reverse_path: String,
    pub forward_paths: Vec<String>,
    pub data: Vec<String>,
}