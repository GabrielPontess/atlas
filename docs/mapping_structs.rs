use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Write, BufWriter};
use std::path::{Path, PathBuf};
use log::{info, warn, error, debug, LevelFilter};

#[derive(Debug)]
enum FileNode {
    File { name: String, extensao: Option<String> },
    Directory { name: String, children: Vec<FileNode> },
}

#[derive(Default, Debug)]
struct RelatorioMetricas {
    total_arquivos: u32,
    total_diretorios: u32,
    por_extensao: BTreeMap<String, u32>,
}

fn main() {
    // CONFIGURAÇÃO DOS LOGS POR PADRÃO:
    // Cria um construtor de logs, define o nível padrão como 'Info' e lê do ambiente (se existir)
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info) // Define 'Info' como padrão se RUST_LOG não estiver configurada
        .parse_default_env()             // Permite sobrescrever via variável de ambiente RUST_LOG se o utilizador quiser
        .init();

    info!("Iniciando varredura (logs ativos por padrão)...");

    let caminho_inicial = r"C:\Users\gabriel.pontes\Documents"; 
    let path = Path::new(caminho_inicial);
    let caminho_absoluto = path.canonicalize().unwrap_or(path.to_path_buf());

    let nome_arquivo_saida = "estrutura_diretorios.md";

    let arquivo = match File::create(nome_arquivo_saida) {
        Ok(f) => f,
        Err(e) => {
            error!("Não foi possível criar o arquivo de saída '{}': {}", nome_arquivo_saida, e);
            return;
        }
    };
    let mut escritor = BufWriter::new(arquivo);
    let mut metricas = RelatorioMetricas::default();

    if let Some(arvore) = mapear_diretorio(path, &mut metricas) {
        info!("Mapeamento concluído com sucesso. Gravando dados em disco...");

        let _ = writeln!(escritor, "# Mapeamento do Diretório: `{}`\n", caminho_absoluto.display());
        let _ = writeln!(escritor, "## 🌳 Árvore de Arquivos\n");
        
        imprimir_markdown_arquivo(&arvore, 0, &mut escritor);

        let _ = writeln!(escritor, "\n---\n");
        let _ = writeln!(escritor, "## 📊 Relatório Estatístico Quantitativo\n");
        let _ = writeln!(escritor, "* **Total de Diretórios:** {}", metricas.total_diretorios);
        let _ = writeln!(escritor, "* **Total de Arquivos:** {}\n", metricas.total_arquivos);

        let _ = writeln!(escritor, "### 📑 Quantitativo por Extensão/Formato\n");
        let _ = writeln!(escritor, "| Extensão / Formato | Quantidade de Arquivos |");
        let _ = writeln!(escritor, "| :--- | :--- |");
        
        for (ext, qtd) in &metricas.por_extensao {
            let _ = writeln!(escritor, "| `{}` | {} |", ext, qtd);
        }

        if escritor.flush().is_ok() {
            info!("Relatório persistido com sucesso em '{}'!", nome_arquivo_saida);
        }
    } else {
        error!("Erro crítico ao mapear o diretório raiz.");
    }
}

fn mapear_diretorio(path: &Path, metricas: &mut RelatorioMetricas) -> Option<FileNode> {
    let name = path.file_name()?.to_string_lossy().into_owned();

    if path.is_dir() {
        metricas.total_diretorios += 1;
        let mut children = Vec::new();
        
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(child_node) = mapear_diretorio(&entry.path(), metricas) {
                    children.push(child_node);
                }
            }
        }
        
        children.sort_by(|a, b| match (a, b) {
            (FileNode::Directory { name: na, .. }, FileNode::Directory { name: nb, .. }) => na.cmp(nb),
            (FileNode::Directory { .. }, FileNode::File { .. }) => std::cmp::Ordering::Less,
            (FileNode::File { .. }, FileNode::Directory { .. }) => std::cmp::Ordering::Greater,
            (FileNode::File { name: na, .. }, FileNode::File { name: nb, .. }) => na.cmp(nb),
        });

        Some(FileNode::Directory { name, children })
    } else {
        metricas.total_arquivos += 1;
        
        let extensao = path.extension()
            .map(|ext| format!(".{}", ext.to_string_lossy().to_lowercase()))
            .or_else(|| Some("[Sem Extensão / Outros]".to_string()));

        if let Some(ref ext) = extensao {
            *metricas.por_extensao.entry(ext.clone()).or_insert(0) += 1;
        }

        Some(FileNode::File { name, extensao })
    }
}

fn imprimir_markdown_arquivo<W: Write>(node: &FileNode, nivel: usize, escritor: &mut W) {
    let indentacao = "  ".repeat(nivel);
    match node {
        FileNode::File { name, .. } => {
            let _ = writeln!(escritor, "{}* 📄 {}", indentacao, name);
        }
        FileNode::Directory { name, children } => {
            let _ = writeln!(escritor, "{}* 📁 **{}/**", indentacao, name);
            for child in children {
                imprimir_markdown_arquivo(child, nivel + 1, escritor);
            }
        }
    }
}