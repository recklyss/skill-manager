use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;

use crate::error::ApiResult;
use crate::marketplace::{
    enrich_skill_item, enrich_skill_marketplace_payload, install_skill as install_marketplace_skill,
};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/install", post(install_skill))
        .route("/popular", get(popular_skills))
        .route("/search", get(search_skills))
        .route("/items/:item_id/document", get(skill_document))
        .route("/items/:item_id", get(skill_detail))
        .route("/mcp/popular", get(popular_mcp))
        .route("/mcp/search", get(search_mcp))
        .route("/mcp/items/:qualified_name", get(mcp_detail))
        .route("/clis/popular", get(popular_clis))
        .route("/clis/search", get(search_clis))
        .route("/clis/items/:slug", get(cli_detail))
}

#[derive(Deserialize)]
struct PageQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    remote: Option<bool>,
    verified: Option<bool>,
}

#[derive(Deserialize)]
struct InstallMarketplaceSkillRequest {
    #[serde(rename = "installToken")]
    install_token: String,
}

async fn install_skill(
    State(state): State<AppState>,
    Json(body): Json<InstallMarketplaceSkillRequest>,
) -> ApiResult<Json<Value>> {
    Ok(Json(install_marketplace_skill(
        &body.install_token,
        state.skills_queries.read_models(),
        state.skills_queries.source_fetcher(),
    )?))
}

async fn popular_skills(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> ApiResult<Json<Value>> {
    let payload = state
        .marketplace
        .skills
        .popular_page(query.limit, query.offset.unwrap_or(0))
        .await?;
    Ok(Json(enrich_skill_marketplace_payload(
        state.skills_queries.read_models(),
        payload,
    )))
}

async fn search_skills(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Json<Value>> {
    let payload = state
        .marketplace
        .skills
        .search_page(
            query.q.as_deref().unwrap_or(""),
            query.limit,
            query.offset.unwrap_or(0),
        )
        .await?;
    Ok(Json(enrich_skill_marketplace_payload(
        state.skills_queries.read_models(),
        payload,
    )))
}

async fn skill_detail(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> ApiResult<Json<Value>> {
    let mut payload = state.marketplace.skills.item_detail(&item_id).await?;
    enrich_skill_item(state.skills_queries.read_models(), &mut payload);
    Ok(Json(payload))
}

async fn skill_document(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        state.marketplace.skills.item_document(&item_id).await?,
    ))
}

async fn popular_mcp(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        state
            .marketplace
            .mcp
            .popular_page(query.limit, query.offset.unwrap_or(0))
            .await?,
    ))
}

async fn search_mcp(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        state
            .marketplace
            .mcp
            .search_page(
                query.q.as_deref().unwrap_or(""),
                query.limit,
                query.offset.unwrap_or(0),
                query.remote,
                query.verified,
            )
            .await?,
    ))
}

async fn mcp_detail(
    State(state): State<AppState>,
    Path(qualified_name): Path<String>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        state.marketplace.mcp.detail(&qualified_name).await?,
    ))
}

async fn popular_clis(
    State(state): State<AppState>,
    Query(query): Query<PageQuery>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        state
            .marketplace
            .clis
            .popular_page(query.limit, query.offset.unwrap_or(0))
            .await?,
    ))
}

async fn search_clis(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Json<Value>> {
    Ok(Json(
        state
            .marketplace
            .clis
            .search_page(
                query.q.as_deref().unwrap_or(""),
                query.limit,
                query.offset.unwrap_or(0),
            )
            .await?,
    ))
}

async fn cli_detail(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<Value>> {
    Ok(Json(state.marketplace.clis.detail(&slug).await?))
}
