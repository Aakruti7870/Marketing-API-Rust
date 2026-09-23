use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

impl PaginationQuery {
    pub fn offset(&self) -> i64 {
        let page = self.page.unwrap_or(1).max(1);
        let limit = self.limit.unwrap_or(20).clamp(1, 100);
        (page - 1) * limit
    }

    pub fn limit(&self) -> i64 {
        self.limit.unwrap_or(20).clamp(1, 100)
    }

    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationMeta {
    pub fn new(total: i64, page: i64, limit: i64) -> Self {
        let total_pages = if limit > 0 { (total as f64 / limit as f64).ceil() as i64 } else { 1 };
        Self {
            page,
            limit,
            total,
            total_pages: total_pages.max(1),
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}
