mod models;
mod scanner;

use std::env;
use std::path::PathBuf;
use std::process;

use scanner::scan_directory;

fn main() {
    let input = resolve_input_path();

    match scan_directory(&input) {
        Ok((_tree, metrics)) => {
            println!("Atlas scanner finalizado.");
            println!("Origem analisada: {}", input.display());
            println!("Diretorios: {}", metrics.total_directories);
            println!("Arquivos: {}", metrics.total_files);

            if metrics.by_extension.is_empty() {
                println!("Extensões: nenhuma encontrada");
            } else {
                println!("Extensões:");
                for (extension, count) in &metrics.by_extension {
                    println!("  {extension}: {count}");
                }
            }
        }
        Err(error) => {
            eprintln!(
                "Falha ao mapear o diretorio '{}': {error}",
                input.display()
            );
            process::exit(1);
        }
    }
}

fn resolve_input_path() -> PathBuf {
    env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("Não foi possível obter o diretório atual"))
}
