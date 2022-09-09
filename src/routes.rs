use axum::{
    extract::{Json, Path},
    http::StatusCode,
    Extension,
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::{
    create_image::execute_insert_image,
    error::ServerError,
    imagga_client::get_tags_for_url,
    query_images::{self, query_image_by_id, query_images, ImageResult},
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
) -> Result<Json<ImageResult>, ServerError> {
    let tags = if request.object_detection {
        get_tags_for_url(&request.image_url)?
    } else {
        vec![]
    };

    match execute_insert_image(request.image_url, tags, request.label, db).await {
        Ok(image_id) => Ok(Json(query_image_by_id(image_id, db).await?)),
        Err(err) => Err(ServerError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to insert image: {err}"),
        )),
    }
}

pub async fn get_image_by_id(
    Path(image_id): Path<i32>,
    Extension(ref db): Extension<DatabaseConnection>,
) -> Result<axum::Json<ImageResult>, ServerError> {
    Ok(Json(query_image_by_id(image_id, db).await?))
}

pub async fn get_all_images(
    Extension(ref db): Extension<DatabaseConnection>,
) -> Result<axum::Json<Vec<ImageResult>>, ServerError> {
    Ok(Json(query_images(db).await?))
}
