pub mod check_results;
pub mod endpoints;

#[derive(serde::Deserialize)]
pub struct Pagination {
    pub limit: Option<u16>,
    pub offset: Option<u16>,
}
