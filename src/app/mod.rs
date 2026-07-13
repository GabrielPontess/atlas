use std::path::PathBuf;

use crate::cli::validate_input_directory;
use crate::models::MappingReport;
use crate::scanner::scan_directory;
use crate::server::run as run_server;

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub input: PathBuf,
    pub port: u16,
    pub open_browser: bool,
}

#[derive(Debug, Clone)]
pub struct SessionStarted {
    pub report: MappingReport,
    pub local_url: String,
    pub browser_error: Option<String>,
}

pub fn build_report(config: &SessionConfig) -> Result<MappingReport, String> {
    validate_input_directory(&config.input)?;

    let scan_result = scan_directory(&config.input)
        .map_err(|error| format!("falha ao mapear o diretorio '{}': {error}", config.input.display()))?;

    Ok(MappingReport::from_scan(
        &config.input,
        scan_result.tree,
        scan_result.metrics,
        scan_result.warnings,
    ))
}

pub async fn start_server_session(
    report: MappingReport,
    port: u16,
    open_browser: bool,
) -> Result<SessionStarted, String> {
    let local_address = run_server(report.clone(), port)
        .await
        .map_err(|error| format!("falha ao iniciar o servidor local: {error}"))?;
    let local_url = format!("http://{local_address}");

    let browser_error = if open_browser {
        open::that(&local_url).err().map(|error| error.to_string())
    } else {
        None
    };

    Ok(SessionStarted {
        report,
        local_url,
        browser_error,
    })
}

pub async fn start_session(config: &SessionConfig) -> Result<SessionStarted, String> {
    let report = build_report(config)?;
    start_server_session(report, config.port, config.open_browser).await
}
