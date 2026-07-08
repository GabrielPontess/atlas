pub mod routes;

use std::io;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;

use crate::models::MappingReport;

#[derive(Clone)]
pub struct AppState {
    pub report: Arc<MappingReport>,
}

pub fn app(report: MappingReport) -> Router {
    let state = AppState {
        report: Arc::new(report),
    };

    routes::router(state)
}

pub async fn run(report: MappingReport, port: u16) -> io::Result<SocketAddr> {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let listener = TcpListener::bind(address).await?;
    let local_address = listener.local_addr()?;
    let app = app(report);

    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            eprintln!("Falha no servidor HTTP: {error}");
        }
    });

    Ok(local_address)
}
