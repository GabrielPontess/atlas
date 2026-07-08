use crate::models::MappingReport;

pub fn render_index(report: &MappingReport) -> String {
    let tree_json = json_for_script_tag(&report.tree);
    let title = format!(
        "Mapeamento do Diretorio <code>{}</code>",
        escape_html(&report.source)
    );

    format!(
        "<!DOCTYPE html>\
<html lang=\"pt-BR\">\
<head>\
  <meta charset=\"utf-8\">\
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
  <title>Atlas Mapper</title>\
  <link rel=\"stylesheet\" href=\"/assets/style.css\">\
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
  <script src=\"/assets/app.js\"></script>\
</body>\
</html>",
        escape_html(&report.source),
        escape_html(&report.generated_at),
        report.summary.total_directories,
        report.summary.total_files,
        render_extensions(&report.summary.by_extension),
        title,
        tree_json
    )
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
    use super::render_index;
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
    }
}
