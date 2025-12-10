#![warn(clippy::all)]

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse};
use axum::routing::{get, put};
use axum::{Json, Router};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use study_test::{handle_submit, oauth_redirect, render_finish_page, render_question, render_redirect, serve_file, try_init_discord, OauthRedirectUrlParams, ServiceState, UserCookie, FIVE_MINUTES, QUESTIONS};
use tokio::net::TcpListener;
use tokio::time::sleep;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let pre_rendered_questions: Vec<_> = QUESTIONS
        .iter()
        .enumerate()
        .map(|(i, v)| render_question(v, i))
        .collect();
    let pre_rendered_redirect = render_redirect();

    let discord_data = try_init_discord().await;
    if discord_data.is_none() {
        warn!("Discord support disabled!");
    }
    let pre_rendered_finish_page = render_finish_page(discord_data.clone());
    let state = ServiceState {
        pre_rendered_questions,
        pre_rendered_redirect,
        discord_data,
        pre_rendered_finish_page,
    };
    let state = Arc::new(state);

    if let Some(discord_data) = state.discord_data.as_ref() {
        let discord_data = discord_data.clone();

        let oauth_state_cleaner = async move || {
            let discord_data = discord_data;
            loop {
                let cutoff = Instant::now() - FIVE_MINUTES;
                {
                    let mut writer = discord_data.oauth_attempts.write().await;
                    writer.retain(|_, &mut (requested_time, _)| requested_time >= cutoff);
                }
                sleep(Duration::from_mins(1)).await;
            }
        };

        tokio::spawn(oauth_state_cleaner());
    }

    let app = Router::new()
        .route(
            "/",
            get(async |State(state): State<Arc<ServiceState>>| {
                state.pre_rendered_redirect.clone().into_response()
            }),
        )
        .route("/favicon.png", get(|| serve_file("favicon.png")))
        .route("/Miracode.ttf", get(|| serve_file("Miracode.ttf")))
        .route(
            "/PretendardVariable.woff2",
            get(|| serve_file("PretendardVariable.woff2")),
        )
        .route(
            "/finish",
            get(async |State(state): State<Arc<ServiceState>>| {
                state.pre_rendered_finish_page.clone().into_response()
            }),
        )
        .route(
            "/submit",
            put(async |State(state): State<Arc<ServiceState>>, Json(cookie): Json<UserCookie>| {
                match state.discord_data.as_ref() {
                    None => StatusCode::NOT_FOUND.into_response(),
                    Some(v) => handle_submit(v.clone(), cookie).await
                }
            }),
        )
        .route(
            "/oauth-redirect",
            get(
                async |param: Query<OauthRedirectUrlParams>, State(state): State<Arc<ServiceState>>| {
                    match state.discord_data.as_ref() {
                        None => StatusCode::NOT_FOUND.into_response(),
                        Some(v) => oauth_redirect(param.0, v.clone()).await.into_response()
                    }

                },
            ),
        )
        // .route("/submit", put())
        .route(
            "/{question}",
            get(
                |State(state): State<Arc<ServiceState>>, path: Path<usize>| {
                    let index = path.0;
                    async move {
                        match state.pre_rendered_questions.get(index) {
                            Some(v) => v.clone().into_response(),
                            None => {
                                StatusCode::NOT_FOUND.into_response()
                            }
                        }
                    }
                },
            ),
        )
        .layer(TraceLayer::new_for_http());

    let args: Vec<String> = std::env::args().collect();
    let address: SocketAddrV4 = args
        .get(1)
        .map(|v| SocketAddrV4::from_str(v.as_str()).expect("Invalid Socket Address"))
        .unwrap_or(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 8080));
    info!("Serving at {}", address);
    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app.with_state(state)).await.unwrap();
}
