use std::cmp::Ordering;
use std::fs;
use std::io;
use std::path::Path;

use crate::models::{FileNode, RelatorioMetricas, ScanWarning};

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub tree: FileNode,
    pub metrics: RelatorioMetricas,
    pub warnings: Vec<ScanWarning>,
}

pub fn scan_directory(path: &Path) -> io::Result<ScanResult> {
    let mut metrics = RelatorioMetricas::default();
    let mut warnings = Vec::new();
    let root = scan_path(path, &mut metrics, &mut warnings, true)?
        .ok_or_else(|| io::Error::other("falha ao mapear o diretorio raiz"))?;

    Ok(ScanResult {
        tree: root,
        metrics,
        warnings,
    })
}

fn scan_path(
    path: &Path,
    metrics: &mut RelatorioMetricas,
    warnings: &mut Vec<ScanWarning>,
    is_root: bool,
) -> io::Result<Option<FileNode>> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if is_root => return Err(error),
        Err(error) => {
            register_warning(metrics, warnings, path, "read_metadata", error);
            return Ok(None);
        }
    };
    let name = node_name(path);

    if metadata.is_dir() {
        metrics.register_directory();

        let mut children = Vec::new();
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(error) if is_root => return Err(error),
            Err(error) => {
                register_warning(metrics, warnings, path, "read_directory", error);
                return Ok(None);
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    register_warning(metrics, warnings, path, "read_entry", error);
                    continue;
                }
            };

            if let Some(child) = scan_path(&entry.path(), metrics, warnings, false)? {
                children.push(child);
            }
        }

        children.sort_by(compare_nodes);
        return Ok(Some(FileNode::directory(name, children)));
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext.to_ascii_lowercase()));

    metrics.register_file(extension.as_deref());
    Ok(Some(FileNode::file(name, extension)))
}

fn register_warning(
    metrics: &mut RelatorioMetricas,
    warnings: &mut Vec<ScanWarning>,
    path: &Path,
    kind: &str,
    error: io::Error,
) {
    metrics.register_warning();
    warnings.push(ScanWarning::new(
        path.display().to_string(),
        kind,
        error.to_string(),
    ));
}

fn node_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string())
}

fn compare_nodes(left: &FileNode, right: &FileNode) -> Ordering {
    match (left, right) {
        (FileNode::Directory { name: left, .. }, FileNode::Directory { name: right, .. }) => {
            left.cmp(right)
        }
        (FileNode::Directory { .. }, FileNode::File { .. }) => Ordering::Less,
        (FileNode::File { .. }, FileNode::Directory { .. }) => Ordering::Greater,
        (FileNode::File { name: left, .. }, FileNode::File { name: right, .. }) => left.cmp(right),
    }
}

#[cfg(test)]
mod tests {
    use super::{compare_nodes, register_warning};
    use crate::models::FileNode;
    use crate::models::RelatorioMetricas;
    use std::io;
    use std::path::Path;

    fn node_name(node: &FileNode) -> &str {
        match node {
            FileNode::File { name, .. } | FileNode::Directory { name, .. } => name,
        }
    }

    #[test]
    fn sorts_directories_before_files() {
        let mut nodes = vec![
            FileNode::file("b.txt".to_string(), Some(".txt".to_string())),
            FileNode::directory("b".to_string(), vec![]),
            FileNode::file("a.txt".to_string(), Some(".txt".to_string())),
            FileNode::directory("a".to_string(), vec![]),
        ];

        nodes.sort_by(compare_nodes);

        assert_eq!(node_name(&nodes[0]), "a");
        assert_eq!(node_name(&nodes[1]), "b");
        assert_eq!(node_name(&nodes[2]), "a.txt");
        assert_eq!(node_name(&nodes[3]), "b.txt");
    }

    #[test]
    fn warning_registration_increments_ignored_items() {
        let mut metrics = RelatorioMetricas::default();
        let mut warnings = Vec::new();

        register_warning(
            &mut metrics,
            &mut warnings,
            Path::new("D:/Acervo/Restrito"),
            "read_directory",
            io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
        );

        assert_eq!(metrics.ignored_items, 1);
        assert_eq!(metrics.warning_count, 1);
        assert_eq!(warnings.len(), 1);
    }
}
