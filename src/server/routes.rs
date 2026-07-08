use axum::http::header;
use axum::extract::State;
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};

use crate::models::{FileNode, RelatorioMetricas};
use crate::render::html::render_index;
use crate::server::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/assets/style.css", get(style_css))
        .route("/assets/app.js", get(app_js))
        .route("/api/tree", get(tree))
        .route("/api/summary", get(summary))
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
