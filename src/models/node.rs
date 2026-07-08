use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FileNode {
    File {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
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
