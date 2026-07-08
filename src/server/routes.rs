use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};

use crate::models::{FileNode, RelatorioMetricas};
use crate::server::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/tree", get(tree))
        .route("/api/summary", get(summary))
        .with_state(state)
}

async fn index(State(state): State<AppState>) -> Html<String> {
    let report = &state.report;

    Html(format!(
        "<!DOCTYPE html>\
<html lang=\"pt-BR\">\
<head>\
  <meta charset=\"utf-8\">\
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
  <title>Atlas Mapper</title>\
  <style>\
    body {{ font-family: Segoe UI, Arial, sans-serif; margin: 0; background: #0f172a; color: #e2e8f0; }}\
    main {{ max-width: 960px; margin: 0 auto; padding: 32px 20px 48px; }}\
    .panel {{ background: #111827; border: 1px solid #334155; border-radius: 16px; padding: 20px; margin-top: 20px; }}\
    h1, h2 {{ margin-top: 0; }}\
    code {{ color: #93c5fd; }}\
    ul {{ padding-left: 20px; }}\
    a {{ color: #93c5fd; }}\
  </style>\
</head>\
<body>\
  <main>\
    <h1>Atlas Mapper iniciado.</h1>\
    <div class=\"panel\">\
      <p><strong>Origem analisada:</strong><br><code>{}</code></p>\
      <p><strong>Gerado em:</strong><br><code>{}</code></p>\
    </div>\
    <div class=\"panel\">\
      <h2>Resumo inicial</h2>\
      <ul>\
        <li>Total de diretorios: {}</li>\
        <li>Total de arquivos: {}</li>\
      </ul>\
    </div>\
    <div class=\"panel\">\
      <h2>Rotas disponiveis</h2>\
      <ul>\
        <li><a href=\"/api/tree\">/api/tree</a></li>\
        <li><a href=\"/api/summary\">/api/summary</a></li>\
      </ul>\
    </div>\
  </main>\
</body>\
</html>",
        report.source,
        report.generated_at,
        report.summary.total_directories,
        report.summary.total_files
    ))
}

async fn tree(State(state): State<AppState>) -> Json<FileNode> {
    Json(state.report.tree.clone())
}

async fn summary(State(state): State<AppState>) -> Json<RelatorioMetricas> {
    Json(state.report.summary.clone())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    use super::router;
    use crate::models::{FileNode, MappingReport, RelatorioMetricas};
    use crate::server::AppState;

    fn test_router() -> axum::Router {
        let report = MappingReport::new(
            std::path::Path::new("D:/Acervo"),
            RelatorioMetricas::default(),
            FileNode::directory("Acervo".to_string(), vec![]),
        );

        router(AppState {
            report: std::sync::Arc::new(report),
        })
    }

    #[tokio::test]
    async fn index_route_returns_ok() {
        let response = test_router()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn tree_route_returns_ok() {
        let response = test_router()
            .oneshot(Request::builder().uri("/api/tree").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn summary_route_returns_ok() {
        let response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/api/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
