
#[derive(Default, Clone)]
pub struct Message {
    pub reverse_path: String,
    pub forward_paths: Vec<String>,
    pub data: Vec<String>,
}

impl Message {
    pub fn undeliverable(reasons: Vec<String>, reverse_path: &str) -> Self {
        Self {
            reverse_path: "<>".to_string(),
            forward_paths: vec![reverse_path.to_string()],
            data: reasons,
        }
    }
}