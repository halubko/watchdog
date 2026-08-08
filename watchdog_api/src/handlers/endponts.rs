use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use uuid::Uuid;
use watchdog_core::{EndpointRepo, RepoError};

use crate::{
    app::AppState,
    dto::{
        Pagination,
        endpoints::{CreateEndpointRequest, EndpointResponse},
    },
    error::ApiError,
};

pub async fn create_endpoint(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateEndpointRequest>,
) -> Result<(StatusCode, Json<EndpointResponse>), ApiError> {
    let endpoint = state.endpoints.create(request.try_into()?).await?;

    Ok((StatusCode::CREATED, Json(endpoint.into())))
}

pub async fn get_by_id_endpoint(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<EndpointResponse>), ApiError> {
    let endpoint = state
        .endpoints
        .get_by_id(id)
        .await?
        .ok_or(ApiError::Repo(RepoError::NotFound))?;

    Ok((StatusCode::OK, Json(endpoint.into())))
}

pub async fn get_list_endpoint(
    State(state): State<Arc<AppState>>,
    Query(pagination): Query<Pagination>,
) -> Result<(StatusCode, Json<Vec<EndpointResponse>>), ApiError> {
    let endpoints = state
        .endpoints
        .list(pagination.limit, pagination.offset)
        .await?
        .into_iter()
        .map(|endpoint| endpoint.into())
        .collect();

    Ok((StatusCode::OK, Json(endpoints)))
}

pub async fn delete_endpoint(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.endpoints.delete(id).await?;

    Ok(StatusCode::NO_CONTENT)
}
