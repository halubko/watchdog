use std::time::Duration;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone)]
pub struct Endpoint {
    pub id: Uuid,
    pub url: String,
    pub interval: u32,
    pub timeout: Option<Duration>,
    pub expected_status: u16,
}

pub struct NewEndpoint {
    pub url: String,
    pub interval: u32,
    pub timeout: Option<Duration>,
    pub expected_status: u16,
}

#[derive(PartialEq, Debug)]
pub struct CheckResult {
    pub id: Uuid,
    pub endpoint_id: Uuid,
    pub date: DateTime<Utc>,
    pub status: CheckStatus,
}

pub struct NewCheckResult {
    pub endpoint_id: Uuid,
    pub status: CheckStatus,
}

#[derive(PartialEq, Debug)]
pub enum CheckStatus {
    Success { status_code: u16, latency_ms: u64 },
    UnexpectedStatus { status_code: u16, latency_ms: u64 },
    Fail { reason: FailureReason },
}

#[derive(PartialEq, Eq, Debug)]

pub enum FailureReason {
    Timeout,
    ConnectionRefused,
    DnsResolution,
    Other(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("endpont not found")]
    NotFound,
    #[error("database error: {0}")]
    Database(String),
}

pub trait EndpointRepo {
    fn create(
        &self,
        new_endpoint: NewEndpoint,
    ) -> impl Future<Output = Result<Endpoint, RepoError>> + Send;

    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<Endpoint>, RepoError>> + Send;

    fn list(
        &self,
        limit: u16,
        offset: u16,
    ) -> impl Future<Output = Result<Vec<Endpoint>, RepoError>> + Send;

    fn delete(&self, endpoint_id: Uuid) -> impl Future<Output = Result<(), RepoError>> + Send;
}

pub trait CheckResultRepo {
    fn save(
        &self,
        new_check_result: NewCheckResult,
    ) -> impl Future<Output = Result<CheckResult, RepoError>> + Send;

    fn list(
        &self,
        endpoint_id: Uuid,
        limit: u16,
        offset: u16,
    ) -> impl Future<Output = Result<Vec<CheckResult>, RepoError>> + Send;

    fn uptime_percentage(
        &self,
        endpoint_id: Uuid,
        since: DateTime<Utc>,
    ) -> impl Future<Output = Result<f64, RepoError>> + Send;
}
