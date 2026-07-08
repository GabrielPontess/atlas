#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileNode {
    File {
        name: String,
        extension: Option<String>,
    },
    Directory {
        name: String,
        children: Vec<FileNode>,
    },
}

impl FileNode {
    pub fn directory(name: String, children: Vec<FileNode>) -> Self {
        Self::Directory { name, children }
    }

    pub fn file(name: String, extension: Option<String>) -> Self {
        Self::File { name, extension }
    }
}
