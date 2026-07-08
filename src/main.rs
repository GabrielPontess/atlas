mod cli;
mod models;
mod scanner;
mod server;

use std::process;

use cli::{Cli, Commands, ServeArgs, validate_input_directory};
use models::MappingReport;
use scanner::scan_directory;
use server::run as run_server;

#[tokio::main]
async fn main() {
    let cli = Cli::parse_args();

    match cli.command {
        Commands::Serve(args) => run_serve(args).await,
    }
}

async fn run_serve(args: ServeArgs) {
    if let Err(message) = validate_input_directory(&args.input) {
        eprintln!("Erro: {message}");
        process::exit(1);
    }

    match scan_directory(&args.input) {
        Ok((tree, metrics)) => {
            let report = MappingReport::from_scan(&args.input, tree, metrics);
            let local_address = match run_server(report.clone(), args.port).await {
                Ok(address) => address,
                Err(error) => {
                    eprintln!("Falha ao iniciar o servidor local: {error}");
                    process::exit(1);
                }
            };
            let local_url = format!("http://{local_address}");

            println!("Atlas Mapper iniciado.");
            println!();
            println!("Origem analisada:");
            println!("{}", report.source);
            println!();
            println!("Data de geracao:");
            println!("{}", report.generated_at);
            println!();
            println!("Servidor local:");
            println!("{local_url}");

            if args.open {
                if let Err(error) = open::that(&local_url) {
                    eprintln!("Falha ao abrir o navegador automaticamente: {error}");
                }

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

            println!();
            println!("Rotas disponiveis:");
            println!("{local_url}/");
            println!("{local_url}/api/tree");
            println!("{local_url}/api/summary");

            if let Err(error) = tokio::signal::ctrl_c().await {
                eprintln!("Falha ao aguardar encerramento do servidor: {error}");
                process::exit(1);
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
