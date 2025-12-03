use axum::body::Body;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use study_test::{render_question, render_redirect, serve_file, QUESTIONS};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    let pre_rendered_questions: Vec<_> = QUESTIONS
        .iter()
        .enumerate()
        .map(|(i, v)| render_question(v, i))
        .collect();
    let pre_rendered_redirect = render_redirect();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .route("/", get(|| async { pre_rendered_redirect }))
        .route("/favicon.png", get(|| serve_file("favicon.png")))
        .route("/Miracode.ttf", get(|| serve_file("Miracode.ttf")))
        .route(
            "/{question}",
            get(|path: Path<usize>| {
                let index = path.0;
                async move {
                    match pre_rendered_questions.get(index) {
                        Some(v) => v.clone().into_response(),
                        None => {
                            let mut response = Response::new(Body::from("Question not found"));
                            *response.status_mut() = StatusCode::NOT_FOUND;
                            response
                        }
                    }
                }
            }),
        )
        .layer(TraceLayer::new_for_http());

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
