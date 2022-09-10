use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
    Extension,
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::{
    create_image::execute_insert_image,
    error::ServerError,
    imagga_client::{get_tags_for_image, ImageInput},
    query_images::{query_image_by_id, query_images, ImageResult, TagFilter},
};

#[derive(Deserialize)]
pub struct NewImageRequest {
    image_url: Option<String>,
    image_base64: Option<String>,
    label: Option<String>,
    object_detection: bool,
}

pub async fn post_image(
    Json(request): Json<NewImageRequest>,
    Extension(ref db): Extension<DatabaseConnection>,
    Extension(imagga_authorization): Extension<String>,
) -> Result<Json<ImageResult>, ServerError> {
    let image_input = match (request.image_url, request.image_base64) {
        (Some(url), None) => Ok(ImageInput::ImageUrl(url)),
        (None, Some(base64)) => Ok(ImageInput::ImageBase64(base64)),
        (_, _) => Err(ServerError::new(
            StatusCode::BAD_REQUEST,
            "Expected an image URL or base64 encoded image (not both)".into(),
        )),
    }?;

    let tags = if request.object_detection {
        get_tags_for_image(image_input.clone(), imagga_authorization)?
    } else {
        // If no tags were requested, we use an empty tag list
        vec![]
    };

    let image_id = execute_insert_image(image_input, tags, request.label, db).await?;

    Ok(Json(query_image_by_id(image_id, db).await?))
}

pub async fn get_image_by_id(
    Path(image_id): Path<i32>,
    Extension(ref db): Extension<DatabaseConnection>,
) -> Result<axum::Json<ImageResult>, ServerError> {
    Ok(Json(query_image_by_id(image_id, db).await?))
}

#[derive(Deserialize)]
pub struct GetImagesQueryParams {
    objects: Option<String>, // request images containing all objects in a comma-separated list
    some_objects: Option<String> // request images containing 1+ objects in a comma separated list
}
pub async fn get_images(
    query_params: Query<GetImagesQueryParams>,
    Extension(ref db): Extension<DatabaseConnection>,
) -> Result<axum::Json<Vec<ImageResult>>, ServerError> {
    let tag_filter = match (&query_params.objects, &query_params.some_objects) {
        (Some(objects_list), None) => {
            let objects: Vec<String> = objects_list.split(",").map(|s| s.to_owned()).collect();
            Ok(TagFilter::ContainsAllTags(objects))
        },
        (None, Some(objects_list)) => {
            let objects: Vec<String> = objects_list.split(",").map(|s| s.to_owned()).collect();
            Ok(TagFilter::ContainsSomeTags(objects))
        },
        (None, None) => Ok(TagFilter::None),
        (Some(_), Some(_)) => Err(ServerError::new(StatusCode::BAD_REQUEST, 
            "Cannot specify both an objects list and a some_objects list".to_owned())),
    }?;
    Ok(Json(query_images(tag_filter, db).await?))
}
