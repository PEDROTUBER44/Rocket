use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
    Json,
};
use uuid::Uuid;
use serde::Deserialize;

use crate::{
    error::{AppError, Result},
    models::session::Session,
    services::folders as folder_service,
    state::AppState,
};

/// The request payload for creating a folder.
#[derive(Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub description: Option<String>,
    pub parent_folder_id: Option<Uuid>,
}

/// The query parameters for listing folder contents.
#[derive(Deserialize)]
pub struct ListFolderQuery {
    #[serde(default)]
    pub folder_id: Option<Uuid>,
}

/// Creates a new folder.
#[axum::debug_handler]
pub async fn create_folder(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Response> {
    if req.name.is_empty() || req.name.len() > 500 {
        return Err(AppError::Validation(
            "Folder name must be between 1 and 500 characters".to_string(),
        ));
    }

    let folder = folder_service::create_folder(
        &state,
        session.user_id,
        req.parent_folder_id,
        req.name,
        req.description,
    )
    .await?;

    let response = sonic_rs::to_string(&sonic_rs::json!({
        "id": folder.id.to_string(),
        "name": folder.name,
        "parent_folder_id": folder.parent_folder_id.map(|id| id.to_string()),
        "created_at": folder.created_at.to_rfc3339(),
        "message": "Folder created successfully"
    }))
    .unwrap();

    Ok((StatusCode::CREATED, response).into_response())
}

/// Lists the contents of a folder.
#[axum::debug_handler]
pub async fn list_folder_contents(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Query(query): Query<ListFolderQuery>,
) -> Result<Response> {
    let (folders, files) = folder_service::list_folder_contents(
        &state,
        session.user_id,
        query.folder_id,
    )
    .await?;

    let folders_json: Vec<_> = folders
        .into_iter()
        .map(|f| {
            sonic_rs::json!({
                "id": f.id.to_string(),
                "name": f.name,
                "description": f.description,
                "created_at": f.created_at.to_rfc3339()
            })
        })
        .collect();

    let files_json: Vec<_> = files
        .into_iter()
        .map(|f| {
            sonic_rs::json!({
                "id": f.id.to_string(),
                "original_filename": f.original_filename,
                "file_size": f.file_size,
                "mime_type": f.mime_type,
                "uploaded_at": f.uploaded_at.to_rfc3339()
            })
        })
        .collect();

    let response = sonic_rs::to_string(&sonic_rs::json!({
        "folders": folders_json,
        "files": files_json,
        "count": folders_json.len() + files_json.len()
    }))
    .unwrap();

    Ok((StatusCode::OK, response).into_response())
}

/// Gets statistics for a folder.
#[axum::debug_handler]
pub async fn get_folder_stats(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(folder_id): Path<Uuid>,
) -> Result<Response> {
    let stats = folder_service::get_folder_with_stats(
        &state,
        session.user_id,
        folder_id,
    )
    .await?
    .ok_or(AppError::NotFound)?;

    let response = sonic_rs::to_string(&sonic_rs::json!({
        "id": stats.id.to_string(),
        "name": stats.name,
        "description": stats.description,
        "created_at": stats.created_at.to_rfc3339(),
        "file_count": stats.file_count,
        "subfolder_count": stats.subfolder_count,
        "total_size": stats.total_size
    }))
    .unwrap();

    Ok((StatusCode::OK, response).into_response())
}

/// Deletes a folder.
#[axum::debug_handler]
pub async fn delete_folder(
    State(state): State<AppState>,
    Extension(session): Extension<Session>,
    Path(folder_id): Path<Uuid>,
) -> Result<Response> {
    folder_service::delete_folder(&state, session.user_id, folder_id).await?;
    Ok((StatusCode::OK, r#"{"message":"Folder deleted successfully"}"#).into_response())
}
