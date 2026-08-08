pub mod check_results;
pub mod endpoints;

#[derive(serde::Deserialize)]
pub struct Pagination {
    pub limit: u16,
    pub offset: u16,
}
