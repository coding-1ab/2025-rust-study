use axum::Router;
use axum::body::Body;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use study_test::{QUESTIONS, render_question, render_redirect, serve_file};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

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
            "/PretendardVariable.woff2",
            get(|| serve_file("PretendardVariable.woff2")),
        )
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

    let args: Vec<String> = std::env::args().collect();
    let address: SocketAddrV4 = args
        .get(1)
        .map(|v| SocketAddrV4::from_str(v.as_str()).expect("Invalid Socket Address"))
        .unwrap_or(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8080));
    info!("Serving at {}", address);
    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
