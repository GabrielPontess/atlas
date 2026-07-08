mod cli;
mod models;
mod scanner;

use std::process;

use cli::{Cli, Commands, ServeArgs, validate_input_directory};
use models::MappingReport;
use scanner::scan_directory;

fn main() {
    let cli = Cli::parse_args();

    match cli.command {
        Commands::Serve(args) => run_serve(args),
    }
}

fn run_serve(args: ServeArgs) {
    if let Err(message) = validate_input_directory(&args.input) {
        eprintln!("Erro: {message}");
        process::exit(1);
    }

    match scan_directory(&args.input) {
        Ok((tree, metrics)) => {
            let report = MappingReport::from_scan(&args.input, tree, metrics);

            println!("Atlas scanner finalizado.");
            println!();
            println!("Origem analisada:");
            println!("{}", report.source);
            println!();
            println!("Data de geracao:");
            println!("{}", report.generated_at);
            println!();
            println!("Porta configurada:");
            println!("{}", args.port);

            if args.open {
                println!();
                println!("Abertura automatica do navegador: habilitada");
            }

            println!();
            println!("Metricas iniciais:");
            println!("Diretorios: {}", report.summary.total_directories);
            println!("Arquivos: {}", report.summary.total_files);

            if report.summary.by_extension.is_empty() {
                println!("Extensoes: nenhuma encontrada");
            } else {
                println!("Extensoes:");
                for (extension, count) in &report.summary.by_extension {
                    println!("  {extension}: {count}");
                }
            }
        }
        Err(error) => {
            eprintln!(
                "Falha ao mapear o diretorio '{}': {error}",
                args.input.display()
            );
            process::exit(1);
        }
    }
}
