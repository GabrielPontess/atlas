use crate::models::MappingReport;

pub fn render_report_json(report: &MappingReport) -> String {
    serde_json::to_string_pretty(report).expect("mapping report should serialize to json")
}
