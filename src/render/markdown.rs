use crate::models::{FileNode, MappingReport};

pub fn render_markdown(report: &MappingReport) -> String {
    let mut output = String::new();

    output.push_str(&format!("# Mapeamento do Diretorio: `{}`\n\n", report.source));
    output.push_str("## Arvore de Arquivos\n\n");
    write_markdown_tree(&report.tree, 0, &mut output);
    output.push_str("\n---\n\n");
    output.push_str("## Relatorio Estatistico Quantitativo\n\n");
    output.push_str(&format!("- **Data de geracao:** {}\n", report.generated_at));
    output.push_str(&format!(
        "- **Total de Diretorios:** {}\n",
        report.summary.total_directories
    ));
    output.push_str(&format!(
        "- **Total de Arquivos:** {}\n\n",
        report.summary.total_files
    ));
    output.push_str("### Quantitativo por Extensao/Formato\n\n");
    output.push_str("| Extensao / Formato | Quantidade de Arquivos |\n");
    output.push_str("| :--- | ---: |\n");

    if report.summary.by_extension.is_empty() {
        output.push_str("| `[sem extensao]` | 0 |\n");
    } else {
        for (extension, count) in &report.summary.by_extension {
            output.push_str(&format!("| `{}` | {} |\n", extension, count));
        }
    }

    output
}

fn write_markdown_tree(node: &FileNode, level: usize, output: &mut String) {
    let indentation = "  ".repeat(level);

    match node {
        FileNode::File { name, .. } => {
            output.push_str(&format!("{}* {}\n", indentation, name));
        }
        FileNode::Directory { name, children } => {
            output.push_str(&format!("{}* **{}/**\n", indentation, name));
            for child in children {
                write_markdown_tree(child, level + 1, output);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::render_markdown;
    use crate::models::{FileNode, MappingReport, RelatorioMetricas};

    #[test]
    fn renders_markdown_with_summary_section() {
        let report = MappingReport::new(
            std::path::Path::new("D:/Acervo"),
            RelatorioMetricas::default(),
            FileNode::directory("Acervo".to_string(), vec![]),
        );

        let markdown = render_markdown(&report);

        assert!(markdown.contains("## Arvore de Arquivos"));
        assert!(markdown.contains("## Relatorio Estatistico Quantitativo"));
    }
}
