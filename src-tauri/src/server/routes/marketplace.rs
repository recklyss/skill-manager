use axum::{extract::Path, routing::get, Json, Router};
use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        // Skills marketplace
        .route("/popular", get(empty_page))
        .route("/search", get(empty_page))
        .route("/items/{id}", get(skill_detail))
        .route("/items/{id}/document", get(skill_document))
        // MCP marketplace
        .route("/mcp/popular", get(empty_page))
        .route("/mcp/search", get(empty_page))
        .route("/mcp/items/{name}", get(mcp_detail))
        // CLI marketplace
        .route("/clis/popular", get(empty_page))
        .route("/clis/search", get(empty_page))
        .route("/clis/items/{slug}", get(cli_detail))
}

async fn empty_page() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "items": [],
        "nextOffset": null,
        "hasMore": false,
    }))
}

async fn skill_detail(Path(_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": _id,
        "name": "Unknown",
        "description": "",
        "installs": 0,
        "stars": null,
        "repoLabel": "",
        "repoImageUrl": null,
        "sourceLinks": {
            "repoLabel": "",
            "repoUrl": "",
            "folderUrl": null,
            "skillsDetailUrl": ""
        },
        "installation": { "status": "installable", "installedSkillRef": null },
        "installToken": ""
    }))
}

async fn skill_document(Path(_id): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "unavailable",
        "documentMarkdown": null,
    }))
}

async fn mcp_detail(Path(_name): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "qualifiedName": _name,
        "displayName": _name,
        "description": "",
        "repository": "",
        "homepageUrl": null,
        "connection": null,
        "toolCount": 0,
        "resourceCount": 0,
        "promptCount": 0,
        "tools": [],
        "resources": [],
        "prompts": [],
        "capabilityCounts": { "tools": 0, "resources": 0, "prompts": 0 },
        "installation": { "status": "installable", "managedName": null }
    }))
}

async fn cli_detail(Path(_slug): Path<String>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "slug": _slug,
        "name": _slug,
        "description": "",
        "homepage": "",
        "stars": 0,
        "installs": 0,
        "command": "",
        "platforms": [],
        "categories": [],
        "sourceLinks": { "repoLabel": "", "repoUrl": "", "folderUrl": null },
        "readmeMarkdown": ""
    }))
}
