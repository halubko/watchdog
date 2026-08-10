use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;
use watchdog_core::CheckResultRepo;

use crate::{
    app::AppState,
    dto::{
        Pagination,
        check_results::{CheckResultResponse, UptimePercentageRequest, UptimePercentageResponse},
    },
    error::ApiError,
};

pub async fn get_list_check_results(
    State(state): State<Arc<AppState>>,
    Path(endpoint_id): Path<Uuid>,
    Query(pagination): Query<Pagination>,
) -> Result<(StatusCode, Json<Vec<CheckResultResponse>>), ApiError> {
    let check_results = state
        .check_results
        .list(
            endpoint_id,
            pagination.limit.unwrap_or(u16::MAX),
            pagination.offset.unwrap_or(u16::MAX),
        )
        .await?
        .into_iter()
        .map(|check_result| check_result.into())
        .collect();

    Ok((StatusCode::OK, Json(check_results)))
}

pub async fn get_uptime_percentage(
    State(state): State<Arc<AppState>>,
    Path(endpoint_id): Path<Uuid>,
    Query(query): Query<UptimePercentageRequest>,
) -> Result<(StatusCode, Json<UptimePercentageResponse>), ApiError> {
    let uptime_percentage = state
        .check_results
        .uptime_percentage(endpoint_id, query.since)
        .await?;

    Ok((
        StatusCode::OK,
        Json(UptimePercentageResponse {
            uptime_percentage,
            since: query.since,
        }),
    ))
}
