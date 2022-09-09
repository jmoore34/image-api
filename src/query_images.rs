use axum::http::StatusCode;
use entity::image;
use entity::prelude::*;
use entity::tag;
use sea_orm::prelude::*;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use serde::Serialize;

use crate::error::ServerError;

#[derive(Serialize)]
pub struct ImageResult {
    url: String,
    tags: Vec<String>,
    label: String,
}
pub async fn query_image_by_id(id: i32, db: &DatabaseConnection) -> Result<ImageResult, ServerError> {
    let image: Option<image::Model> = Image::find()
        .filter(image::Column::Id.eq(id))
        .one(db)
        .await?;

    match image {
        None => Err(ServerError::new(
            StatusCode::NOT_FOUND,
            format!("No image found with id {id}"),
        )),
        Some(image) => {
            let tags: Vec<tag::Model> = image.find_related(Tag).all(db).await?;
            // Now extract names from Tags (shadowing old value)
            let tags: Vec<String> = tags.iter().map(|tag| tag.name.clone()).collect();
            Ok(ImageResult {
                url: image.url,
                label: image.label,
                tags,
            })
        }
    }
}

