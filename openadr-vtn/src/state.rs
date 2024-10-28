use crate::{
    data_source::{
        AuthSource, DataSource, EventCrud, ProgramCrud, ReportCrud, ResourceCrud, VenCrud,
    },
    error::AppError,
    jwt::JwtManager,
};
use axum::{
    extract::{FromRef, Request},
    middleware,
    middleware::Next,
    response::IntoResponse,
    routing::{delete, get, post},
};
use openadr_wire::resource::Resource;
use reqwest::StatusCode;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::api::{auth, event, program, report, resource, user, ven};

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
            .route(
                "/vens/:ven_id/resources",
                get(resource::get_all).post(resource::add),
            )
            .route(
                "/vens/:ven_id/resources/:id",
                get(resource::get)
                    .put(resource::edit)
                    .delete(resource::delete),
            )
            .route("/auth/token", post(auth::token))
            .route("/users", get(user::get_all).post(user::add_user))
            .route(
                "/users/:id",
                get(user::get)
                    .put(user::edit)
                    .delete(user::delete_user)
                    .post(user::add_credential),
            )
            .route(
                "/users/:user_id/:client_id",
                delete(user::delete_credential),
            )
            .fallback(handler_404)
            .layer(middleware::from_fn(method_not_allowed))
            .layer(TraceLayer::new_for_http())
            .merge(
                SwaggerUi::new("/docs/ui")
                    .url("/docs/openapi.json", OpenApiDocument::openapi())
            )
    }

    pub fn into_router(self) -> axum::Router {
        Self::router_without_state().with_state(self)
    }
}

async fn method_not_allowed(req: Request, next: Next) -> impl IntoResponse {
    let resp = next.run(req).await;
    let status = resp.status();
    match status {
        StatusCode::METHOD_NOT_ALLOWED => Err(AppError::MethodNotAllowed),
        _ => Ok(resp),
    }
}

async fn handler_404() -> AppError {
    AppError::NotFound
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

impl FromRef<AppState> for Arc<dyn ResourceCrud> {
    fn from_ref(state: &AppState) -> Arc<dyn ResourceCrud> {
        state.storage.resources()
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "OpenADR 3 API",
        version = "3.0.1",
        description = "
    The OpenADR 3.0.0 API supports energy retailer to energy customer Demand Response programs.
    The API includes the following capabilities and operations:

    __Manage programs:__

    * Create/Update/Delete a program
    * Search programs

    __Manage events:__

    * Create/Update/Delete an event
    * Search events

    __Manage reports:__

    * Create/Update/Delete a report
    * Search reports

    __Manage subscriptions:__

    * Create/Update/Delete subscriptions to programs, events, and reports
    * Search subscriptions
    * Subscriptions allows clients to register a callback URL (webhook) to be notified
      on the change of state of a resource

    __Manage vens:__

    * Create/Update/Delete vens and ven resources
    * Search ven and ven resources

    __Manage tokens:__

    * Obtain an access token
    * This endpoint is provided as a convenience and may be neglected in a commercial implementation
"
    ),
    paths(
        resource::get_all,
        resource::get,
        resource::edit,
        resource::delete
    ),
    components(schemas(Resource)),
)]
struct OpenApiDocument;
