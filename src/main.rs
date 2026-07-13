mod app;
mod cli;
mod models;
mod render;
mod scanner;
mod server;
mod tui;

use std::process;

use app::{SessionConfig, start_session};
use cli::{Cli, Commands, ServeArgs};

#[tokio::main]
async fn main() {
    let cli = Cli::parse_args();

    match cli.command {
        Some(Commands::Serve(args)) => run_serve(args).await,
        None => {
            if let Err(error) = tui::run(tokio::runtime::Handle::current().clone()) {
                eprintln!("Falha ao iniciar a TUI: {error}");
                process::exit(1);
            }
        }
    }
}

async fn run_serve(args: ServeArgs) {
    let config = SessionConfig {
        input: args.input,
        port: args.port,
        open_browser: args.open,
    };

    match start_session(&config).await {
        Ok(session) => {
            let local_url = session.local_url;
            let report = session.report;

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

            if config.open_browser {
                println!();
                println!("Abertura automatica do navegador: habilitada");
                if let Some(error) = session.browser_error {
                    eprintln!("Falha ao abrir o navegador automaticamente: {error}");
                }
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

            if report.summary.warning_count > 0 {
                println!();
                println!("Avisos de leitura: {}", report.summary.warning_count);
                println!("Itens ignorados: {}", report.summary.ignored_items);

                for warning in &report.warnings {
                    eprintln!(
                        "[aviso:{}] {} -> {}",
                        warning.kind, warning.path, warning.message
                    );
                }
            }

            println!();
            println!("Rotas disponiveis:");
            println!("{local_url}/");
            println!("{local_url}/api/tree");
            println!("{local_url}/api/summary");
            println!("{local_url}/download/html");
            println!("{local_url}/download/json");
            println!("{local_url}/download/markdown");

            if let Err(error) = tokio::signal::ctrl_c().await {
                eprintln!("Falha ao aguardar encerramento do servidor: {error}");
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("Erro: {error}");
            process::exit(1);
        }
    }
}
