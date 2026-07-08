use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScanWarning {
    pub path: String,
    pub kind: String,
    pub message: String,
}

impl ScanWarning {
    pub fn new(path: String, kind: &str, message: String) -> Self {
        Self {
            path,
            kind: kind.to_string(),
            message,
        }
    }
}
