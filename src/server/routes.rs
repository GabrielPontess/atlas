use axum::http::header;
use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};

use crate::models::{FileNode, RelatorioMetricas};
use crate::render::html::{render_index, render_standalone};
use crate::render::json::render_report_json;
use crate::render::markdown::render_markdown;
use crate::server::AppState;

const DOWNLOAD_BASENAME: &str = "estrutura_diretorios";

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/assets/style.css", get(style_css))
        .route("/assets/app.js", get(app_js))
        .route("/api/tree", get(tree))
        .route("/api/summary", get(summary))
        .route("/download/html", get(download_html))
        .route("/download/json", get(download_json))
        .route("/download/markdown", get(download_markdown))
        .with_state(state)
}

async fn index(State(state): State<AppState>) -> Html<String> {
    Html(render_index(state.report.as_ref()))
}

async fn style_css() -> ([(axum::http::header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        include_str!("../assets/style.css"),
    )
}

async fn app_js() -> ([(axum::http::header::HeaderName, &'static str); 1], &'static str) {
    (
        [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
        include_str!("../assets/app.js"),
    )
}

async fn tree(State(state): State<AppState>) -> Json<FileNode> {
    Json(state.report.tree.clone())
}

async fn summary(State(state): State<AppState>) -> Json<RelatorioMetricas> {
    Json(state.report.summary.clone())
}

async fn download_html(State(state): State<AppState>) -> ([(axum::http::header::HeaderName, String); 2], String) {
    download_response(
        "text/html; charset=utf-8",
        format!("{DOWNLOAD_BASENAME}.html"),
        render_standalone(state.report.as_ref()),
    )
}

async fn download_json(State(state): State<AppState>) -> ([(axum::http::header::HeaderName, String); 2], String) {
    download_response(
        "application/json; charset=utf-8",
        format!("{DOWNLOAD_BASENAME}.json"),
        render_report_json(state.report.as_ref()),
    )
}

async fn download_markdown(State(state): State<AppState>) -> ([(axum::http::header::HeaderName, String); 2], String) {
    download_response(
        "text/markdown; charset=utf-8",
        format!("{DOWNLOAD_BASENAME}.md"),
        render_markdown(state.report.as_ref()),
    )
}

fn download_response(content_type: &str, filename: String, body: String) -> ([(axum::http::header::HeaderName, String); 2], String) {
    (
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        body,
    )
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::header;
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
    async fn asset_routes_return_ok() {
        let css_response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/assets/style.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let js_response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/assets/app.js")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(css_response.status(), StatusCode::OK);
        assert_eq!(js_response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn download_routes_return_ok() {
        let html_response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/download/html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let json_response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/download/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let markdown_response = test_router()
            .oneshot(
                Request::builder()
                    .uri("/download/markdown")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(html_response.status(), StatusCode::OK);
        assert_eq!(json_response.status(), StatusCode::OK);
        assert_eq!(markdown_response.status(), StatusCode::OK);
        assert_eq!(
            json_response.headers().get(header::CONTENT_DISPOSITION).unwrap(),
            "attachment; filename=\"estrutura_diretorios.json\""
        );
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
