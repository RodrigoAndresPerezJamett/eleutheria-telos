// MCP server — Phase 0 skeleton
// Full implementation in Phase 4 (MCP + Plugin Ecosystem).
// These stubs register the routes so the router compiles and the endpoints
// return a clear "not yet implemented" response.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;

pub async fn mcp_sse_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "MCP SSE transport not yet implemented — coming in Phase 4"
        })),
    )
}

pub async fn mcp_post_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "error": "MCP stdio-compatible endpoint not yet implemented — coming in Phase 4"
        })),
    )
}
