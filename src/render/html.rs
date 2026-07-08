use crate::models::MappingReport;

const STYLE_CSS: &str = include_str!("../assets/style.css");
const APP_JS: &str = include_str!("../assets/app.js");

pub fn render_index(report: &MappingReport) -> String {
    render_html_page(report, AssetMode::External)
}

pub fn render_standalone(report: &MappingReport) -> String {
    render_html_page(report, AssetMode::Inline)
}

enum AssetMode {
    External,
    Inline,
}

fn render_html_page(report: &MappingReport, asset_mode: AssetMode) -> String {
    let tree_json = json_for_script_tag(&report.tree);
    let title = format!(
        "Mapeamento do Diretorio <code>{}</code>",
        escape_html(&report.source)
    );
    let head_assets = match asset_mode {
        AssetMode::External => {
            "<link rel=\"stylesheet\" href=\"/assets/style.css\">".to_string()
        }
        AssetMode::Inline => format!("<style>{STYLE_CSS}</style>"),
    };
    let script_assets = match asset_mode {
        AssetMode::External => "<script src=\"/assets/app.js\"></script>".to_string(),
        AssetMode::Inline => format!("<script>{APP_JS}</script>"),
    };
    let controls = render_controls(&asset_mode);

    format!(
        "<!DOCTYPE html>\
<html lang=\"pt-BR\">\
<head>\
  <meta charset=\"utf-8\">\
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
  <title>Atlas Mapper</title>\
  {}\
</head>\
<body>\
  <div class=\"layout\">\
    <aside class=\"panel sidebar\">\
      <h1>Atlas Mapper</h1>\
      <p class=\"subtitle\">Visualizacao tecnica interativa da arvore de diretorios e arquivos mapeada pelo live server local do projeto Atlas.</p>\
      <div class=\"meta-list\">\
        <div class=\"meta-item\">\
          <span class=\"meta-label\">Origem analisada</span>\
          <p class=\"meta-value\"><code>{}</code></p>\
        </div>\
        <div class=\"meta-item\">\
          <span class=\"meta-label\">Data de geracao</span>\
          <p class=\"meta-value\"><code>{}</code></p>\
        </div>\
      </div>\
      <input id=\"search\" class=\"search\" type=\"search\" placeholder=\"Buscar pasta ou arquivo...\">\
      <div class=\"controls\">\
        <button class=\"btn\" id=\"expandAll\" type=\"button\">Expandir tudo</button>\
        <button class=\"btn\" id=\"collapseAll\" type=\"button\">Recolher tudo</button>\
      </div>\
      {}\
      <div class=\"stats\">\
        <div class=\"stat-card\">\
          <span class=\"stat-label\">Pastas</span>\
          <span class=\"stat-value\">{}</span>\
        </div>\
        <div class=\"stat-card\">\
          <span class=\"stat-label\">Arquivos</span>\
          <span class=\"stat-value\">{}</span>\
        </div>\
      </div>\
      <div class=\"legend\">\
        <span class=\"chip\"><span class=\"dot folder\"></span>Pastas</span>\
        <span class=\"chip\"><span class=\"dot file\"></span>Arquivos</span>\
      </div>\
      {}\
      <p class=\"footer-note\">A arvore e navegavel por niveis. Use a busca para filtrar nomes e localizar rapidamente projetos, pastas tecnicas ou arquivos especificos.</p>\
    </aside>\
    <main class=\"panel content\">\
      <div class=\"tree-toolbar\">\
        <div><div class=\"tree-title\">{}</div></div>\
        <div class=\"tree-status\">Interativo</div>\
      </div>\
      <div class=\"tree-wrap\">\
        <ul id=\"treeRoot\" class=\"tree-root\"></ul>\
      </div>\
    </main>\
  </div>\
  <script id=\"tree-data\" type=\"application/json\">{}</script>\
  {}\
</body>\
</html>",
        head_assets,
        escape_html(&report.source),
        escape_html(&report.generated_at),
        controls,
        report.summary.total_directories,
        report.summary.total_files,
        render_extensions(&report.summary.by_extension),
        title,
        tree_json,
        script_assets
    )
}

fn render_controls(asset_mode: &AssetMode) -> &'static str {
    match asset_mode {
        AssetMode::External => {
            "<div class=\"downloads\"><a class=\"btn btn-link\" href=\"/download/html\">Baixar HTML</a><a class=\"btn btn-link\" href=\"/download/json\">Baixar JSON</a><a class=\"btn btn-link\" href=\"/download/markdown\">Baixar Markdown</a></div>"
        }
        AssetMode::Inline => {
            "<div class=\"downloads\"><span class=\"chip\">Arquivo standalone</span></div>"
        }
    }
}

fn render_extensions(extensions: &std::collections::BTreeMap<String, u64>) -> String {
    if extensions.is_empty() {
        return "<div class=\"extensions\"><h2>Extensoes</h2><p class=\"footer-note\">Nenhuma extensao encontrada no acervo.</p></div>".to_string();
    }

    let items = extensions
        .iter()
        .map(|(extension, count)| {
            format!(
                "<li><code>{}</code><span>{}</span></li>",
                escape_html(extension),
                count
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        "<div class=\"extensions\"><h2>Extensoes</h2><ul class=\"extensions-list\">{items}</ul></div>"
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn json_for_script_tag<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value)
        .expect("serializable mapping model")
        .replace("</script", "<\\/script")
}

#[cfg(test)]
mod tests {
    use super::{render_index, render_standalone};
    use crate::models::{FileNode, MappingReport, RelatorioMetricas};

    #[test]
    fn renders_tree_bootstrap_markup() {
        let report = MappingReport::new(
            std::path::Path::new("D:/Acervo"),
            RelatorioMetricas::default(),
            FileNode::directory("Acervo".to_string(), vec![]),
        );

        let html = render_index(&report);

        assert!(html.contains("id=\"treeRoot\""));
        assert!(html.contains("id=\"tree-data\""));
        assert!(html.contains("Expandir tudo"));
        assert!(html.contains("/download/json"));
    }

    #[test]
    fn renders_standalone_without_external_dependencies() {
        let report = MappingReport::new(
            std::path::Path::new("D:/Acervo"),
            RelatorioMetricas::default(),
            FileNode::directory("Acervo".to_string(), vec![]),
        );

        let html = render_standalone(&report);

        assert!(html.contains("<style>"));
        assert!(html.contains("<script>"));
        assert!(!html.contains("/assets/style.css"));
        assert!(!html.contains("/assets/app.js"));
        assert!(!html.contains("/download/json"));
    }
}
