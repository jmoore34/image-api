use axum::{extract::Json, Extension, http::StatusCode};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::{
    create_image::{execute_insert_image},
    error::ServerError,
    imagga_client::get_tags_for_url,
};

#[derive(Deserialize)]
pub struct NewImageRequest {
    image_url: String,
    label: Option<String>,
    object_detection: bool,
}

pub async fn post_image(
    Json(request): Json<NewImageRequest>,
    Extension(ref db): Extension<DatabaseConnection>,
) -> Result<String, ServerError> {
    let tags = if request.object_detection {
        get_tags_for_url(&request.image_url)?
    } else {
        vec![]
    };

    match execute_insert_image(request.image_url, tags, request.label, db).await {
        Ok(image_id) => Ok(format!("Created image with id {image_id}")),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to insert image: {err}"),
        )),
    }
}
