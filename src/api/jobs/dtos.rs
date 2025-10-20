use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use stefn::service::Pagination;
use utoipa::{IntoParams, ToResponse, ToSchema};


#[derive(Deserialize, Serialize, ToSchema, IntoParams)]
pub struct JobFilters {
    /// Filter by country code (e.g., "FR", "US")
    pub country: Option<String>,
    /// Filter by job title or keywords
     pub title: Option<String>,
    /// Filter by company name
     pub company: Option<String>,
    /// Filter by location/city
     pub location: Option<String>,
    /// Filter by contract type (e.g., "full_time", "part_time", "contract")
     pub contract_type: Option<String>,
    /// Filter by minimum number of days since posted
     pub days_since_posted: Option<i32>,
    /// Filter by remote work availability
     pub is_remote: Option<bool>,
    /// Filter by skills (comma-separated)
     pub skills: Option<String>,
     #[serde(flatten)]
     pub pagination: Pagination
}

#[derive(Deserialize, Serialize, ToResponse, ToSchema, FromRow)]
pub struct JobResponse {
     id: String,
     title: String,
     company: String,
     location: Option<String>,
     contract_type: Option<String>,
     description: Option<String>,
     url: String,
     posted_date: Option<String>,
     is_remote: Option<bool>,
     skills: Vec<String>,
     salary_range: Option<String>,
}
