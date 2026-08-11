use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use watchdog_db::{check_result::PgCheckResultRepo, endpoint::PgEndpointRepo};

use crate::{
    handlers::{
        check_results::{get_list_check_results, get_uptime_percentage},
        endponts::{create_endpoint, delete_endpoint, get_by_id_endpoint, get_list_endpoint},
    },
    notifier::Notifier,
};

pub struct AppState {
    pub endpoints: PgEndpointRepo,
    pub check_results: PgCheckResultRepo,
    pub notifier: Arc<Notifier>,
}

impl AppState {
    pub fn new(
        endpoints: PgEndpointRepo,
        check_results: PgCheckResultRepo,
        notifier: Arc<Notifier>,
    ) -> Self {
        Self {
            endpoints,
            check_results,
            notifier,
        }
    }
}

pub fn app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/endpoints", post(create_endpoint).get(get_list_endpoint))
        .route(
            "/endpoints/{id}",
            get(get_by_id_endpoint).delete(delete_endpoint),
        )
        .route("/endpoints/{id}/check_results", get(get_list_check_results))
        .route(
            "/endpoints/{id}/uptime_percentage",
            get(get_uptime_percentage),
        )
        .with_state(state)
}
