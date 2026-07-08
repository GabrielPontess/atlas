use std::path::Path;

use chrono::{Local, SecondsFormat};
use serde::Serialize;

use crate::models::{FileNode, RelatorioMetricas, ScanWarning};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MappingReport {
    pub source: String,
    pub generated_at: String,
    pub summary: RelatorioMetricas,
    pub tree: FileNode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<ScanWarning>,
}

impl MappingReport {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn new(source: &Path, summary: RelatorioMetricas, tree: FileNode) -> Self {
        Self::with_warnings(source, summary, tree, Vec::new())
    }

    pub fn with_warnings(
        source: &Path,
        summary: RelatorioMetricas,
        tree: FileNode,
        warnings: Vec<ScanWarning>,
    ) -> Self {
        Self {
            source: display_path(source),
            generated_at: Local::now().to_rfc3339_opts(SecondsFormat::Secs, false),
            summary,
            tree,
            warnings,
        }
    }

    pub fn from_scan(
        source: &Path,
        tree: FileNode,
        summary: RelatorioMetricas,
        warnings: Vec<ScanWarning>,
    ) -> Self {
        let source = source
            .canonicalize()
            .unwrap_or_else(|_| source.to_path_buf());

        Self::with_warnings(&source, summary, tree, warnings)
    }
}

fn display_path(path: &Path) -> String {
    let rendered = path.display().to_string();
    rendered.strip_prefix(r"\\?\").unwrap_or(&rendered).to_string()
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::MappingReport;
    use crate::models::{FileNode, RelatorioMetricas};

    #[test]
    fn serializes_expected_top_level_fields() {
        let report = MappingReport::new(
            std::path::Path::new("D:/Acervo"),
            RelatorioMetricas::default(),
            FileNode::directory("Acervo".to_string(), vec![]),
        );

        let json: Value = serde_json::to_value(&report).expect("report should serialize");

        assert!(json.get("source").is_some());
        assert!(json.get("generated_at").is_some());
        assert!(json.get("summary").is_some());
        assert!(json.get("tree").is_some());
    }

    #[test]
    fn serializes_summary_with_spec_field_names() {
        let mut summary = RelatorioMetricas::default();
        summary.register_directory();
        summary.register_file(Some(".pdf"));

        let report = MappingReport::new(
            std::path::Path::new("D:/Acervo"),
            summary,
            FileNode::directory("Acervo".to_string(), vec![]),
        );

        let json: Value = serde_json::to_value(&report).expect("report should serialize");
        let summary = json
            .get("summary")
            .expect("summary should exist");

        assert_eq!(summary.get("total_directories").and_then(Value::as_u64), Some(1));
        assert_eq!(summary.get("total_files").and_then(Value::as_u64), Some(1));
        assert_eq!(
            summary
                .get("extensions")
                .and_then(|value| value.get(".pdf"))
                .and_then(Value::as_u64),
            Some(1)
        );
        assert_eq!(summary.get("ignored_items").and_then(Value::as_u64), Some(0));
        assert_eq!(summary.get("warning_count").and_then(Value::as_u64), Some(0));
    }

    #[test]
    fn strips_windows_verbatim_prefix_from_source() {
        let report = MappingReport::new(
            std::path::Path::new(r"\\?\C:\Acervo"),
            RelatorioMetricas::default(),
            FileNode::directory("Acervo".to_string(), vec![]),
        );

        assert_eq!(report.source, r"C:\Acervo");
    }
}
