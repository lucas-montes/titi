use sqlx::{Postgres, QueryBuilder};
use stefn::{database::Database, errors::AppError};

use super::dtos::{JobFilters, JobResponse};

pub async fn count_jobs(database: &Database, payload: &JobFilters) -> Result<i64, AppError> {
    let mut query_builder =
        QueryBuilder::<Postgres>::new("SELECT COUNT(pk) as count FROM jobs WHERE");

    apply_filters(&mut query_builder, payload);

    let count = query_builder
        .build_query_scalar()
        .fetch_one(&**database)
        .await?;

    tracing::debug!(total_count = count, "Counted jobs matching filters");

    Ok(count)
}

fn apply_filters<'a>(query_builder: &mut QueryBuilder<'a, Postgres>, filters: &'a JobFilters) {
    if let Some(country) = &filters.country {
        query_builder.push(" country = ");
        query_builder.push_bind(country);
    }

    if let Some(title) = &filters.title {
        query_builder.push(" AND title ILIKE ");
        query_builder.push_bind(format!("%{}%", title));
    }

    if let Some(company) = &filters.company {
        query_builder.push(" AND company ILIKE ");
        query_builder.push_bind(format!("%{}%", company));
    }

    if let Some(location) = &filters.location {
        query_builder.push(" AND location ILIKE ");
        query_builder.push_bind(format!("%{}%", location));
    }

    if let Some(contract_type) = &filters.contract_type {
        query_builder.push(" AND contract_type = ");
        query_builder.push_bind(contract_type);
    }

    if let Some(days) = filters.days_since_posted {
        query_builder.push(" AND posted_date >= NOW() - INTERVAL '");
        query_builder.push(days.to_string());
        query_builder.push(" days'");
    }

    if let Some(is_remote) = filters.is_remote {
        query_builder.push(" AND is_remote = ");
        query_builder.push_bind(is_remote);
    }

    if let Some(skills) = &filters.skills {
        let skills_vec: Vec<&str> = skills.split(',').map(|s| s.trim()).collect();
        if !skills_vec.is_empty() {
            query_builder.push(" AND skills && ");
            query_builder.push_bind(skills_vec);
        }
    }
}

pub async fn fetch_jobs(
    database: &Database,
    payload: &JobFilters,
) -> Result<Vec<JobResponse>, AppError> {
    let mut query_builder = QueryBuilder::new(
        "SELECT id, title, company, location, contract_type,
                description, url, posted_date, is_remote, skills, salary_range
         FROM jobs WHERE",
    );

    apply_filters(&mut query_builder, payload);

    // Add ordering
    query_builder.push(" ORDER BY posted_date DESC, created_at DESC");

    // Add pagination
    let per_page = payload.pagination.per_page();
    let offset = payload.pagination.offset();

    query_builder.push(" LIMIT ");
    query_builder.push_bind(per_page as i64);
    query_builder.push(" OFFSET ");
    query_builder.push_bind(offset as i64);

    // Execute query
    let rows = query_builder
        .build_query_as()
        .fetch_all(&**database)
        .await?;

    tracing::info!(
        job_count = rows.len(),
        per_page = per_page,
        offset = offset,
        "Successfully fetched jobs"
    );

    Ok(rows)
}
