use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct RelatorioMetricas {
    #[serde(rename = "total_files")]
    pub total_files: u64,
    #[serde(rename = "total_directories")]
    pub total_directories: u64,
    #[serde(rename = "extensions")]
    pub by_extension: BTreeMap<String, u64>,
}

impl RelatorioMetricas {
    pub fn register_directory(&mut self) {
        self.total_directories += 1;
    }

    pub fn register_file(&mut self, extension: Option<&str>) {
        self.total_files += 1;

        let key = extension.unwrap_or("[sem extensao]");
        *self.by_extension.entry(key.to_string()).or_insert(0) += 1;
    }
}
