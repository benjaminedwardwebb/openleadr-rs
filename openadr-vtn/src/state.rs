use crate::api::ven;
use crate::data_source::{AuthSource, DataSource, EventCrud, ProgramCrud, ReportCrud, VenCrud};
use crate::error::AppError;
use crate::jwt::JwtManager;
use axum::extract::{FromRef, Request};
use axum::middleware;
use axum::middleware::Next;
use axum::response::IntoResponse;
use reqwest::StatusCode;
use std::sync::Arc;

#[derive(Clone, FromRef)]
pub struct AppState {
    pub storage: Arc<dyn DataSource>,
    pub jwt_manager: Arc<JwtManager>,
}

impl AppState {
    pub fn new<S: DataSource>(storage: S, jwt_manager: JwtManager) -> Self {
        Self {
            storage: Arc::new(storage),
            jwt_manager: Arc::new(jwt_manager),
        }
    }

    fn router_without_state() -> axum::Router<Self> {
        use axum::routing::{get, post};
        use tower_http::trace::TraceLayer;

        use crate::api::program;
        use crate::api::report;
        use crate::api::{auth, event};

        axum::Router::new()
            .route("/programs", get(program::get_all).post(program::add))
            .route(
                "/programs/:id",
                get(program::get).put(program::edit).delete(program::delete),
            )
            .route("/reports", get(report::get_all).post(report::add))
            .route(
                "/reports/:id",
                get(report::get).put(report::edit).delete(report::delete),
            )
            .route("/events", get(event::get_all).post(event::add))
            .route(
                "/events/:id",
                get(event::get).put(event::edit).delete(event::delete),
            )
            .route("/vens", get(ven::get_all).post(ven::add))
            .route(
                "/vens/:id",
                get(ven::get).put(ven::edit).delete(ven::delete),
            )
            .route("/auth/register", post(auth::register))
            .route("/auth/token", post(auth::token))
            .layer(middleware::from_fn(method_not_allowed))
            .layer(TraceLayer::new_for_http())
    }

    pub fn into_router(self) -> axum::Router {
        Self::router_without_state().with_state(self)
    }
}

pub async fn method_not_allowed(req: Request, next: Next) -> impl IntoResponse {
    let resp = next.run(req).await;
    let status = resp.status();
    match status {
        StatusCode::METHOD_NOT_ALLOWED => Err(AppError::MethodNotAllowed),
        _ => Ok(resp),
    }
}

impl FromRef<AppState> for Arc<dyn AuthSource> {
    fn from_ref(state: &AppState) -> Arc<dyn AuthSource> {
        state.storage.auth()
    }
}

impl FromRef<AppState> for Arc<dyn ProgramCrud> {
    fn from_ref(state: &AppState) -> Arc<dyn ProgramCrud> {
        state.storage.programs()
    }
}

impl FromRef<AppState> for Arc<dyn EventCrud> {
    fn from_ref(state: &AppState) -> Arc<dyn EventCrud> {
        state.storage.events()
    }
}

impl FromRef<AppState> for Arc<dyn ReportCrud> {
    fn from_ref(state: &AppState) -> Arc<dyn ReportCrud> {
        state.storage.reports()
    }
}

impl FromRef<AppState> for Arc<dyn VenCrud> {
    fn from_ref(state: &AppState) -> Arc<dyn VenCrud> {
        state.storage.vens()
    }
}
