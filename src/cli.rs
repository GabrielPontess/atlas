use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};

const DEFAULT_PORT: u16 = 8787;

#[derive(Debug, Parser)]
#[command(name = "atlas")]
#[command(version)]
#[command(about = "Mapeia acervos documentais e prepara a visualizacao local")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Serve(ServeArgs),
}

#[derive(Debug, Args)]
pub struct ServeArgs {
    #[arg(long, value_name = "PATH")]
    pub input: PathBuf,

    #[arg(long, default_value_t = DEFAULT_PORT)]
    pub port: u16,

    #[arg(long)]
    pub open: bool,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

pub fn validate_input_directory(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!(
            "o caminho informado nao existe: {}",
            path.display()
        ));
    }

    if !path.is_dir() {
        return Err(format!(
            "o caminho informado nao e um diretorio: {}",
            path.display()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::validate_input_directory;

    #[test]
    fn rejects_missing_directory() {
        let invalid = Path::new("__atlas_missing_directory__");
        let result = validate_input_directory(invalid);

        assert!(result.is_err());
    }
}
