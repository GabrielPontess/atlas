use std::cmp::Ordering;
use std::fs;
use std::io;
use std::path::Path;

use crate::models::{FileNode, RelatorioMetricas};

pub fn scan_directory(path: &Path) -> io::Result<(FileNode, RelatorioMetricas)> {
    let mut metrics = RelatorioMetricas::default();
    let root = scan_path(path, &mut metrics)?;
    Ok((root, metrics))
}

fn scan_path(path: &Path, metrics: &mut RelatorioMetricas) -> io::Result<FileNode> {
    let metadata = fs::metadata(path)?;
    let name = node_name(path);

    if metadata.is_dir() {
        metrics.register_directory();

        let mut children = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            children.push(scan_path(&entry.path(), metrics)?);
        }

        children.sort_by(compare_nodes);
        return Ok(FileNode::directory(name, children));
    }

    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext.to_ascii_lowercase()));

    metrics.register_file(extension.as_deref());
    Ok(FileNode::file(name, extension))
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
    use super::compare_nodes;
    use crate::models::FileNode;

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
}
