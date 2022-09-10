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
pub async fn query_image_by_id(
    id: i32,
    db: &DatabaseConnection,
) -> Result<ImageResult, ServerError> {
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
pub enum TagFilter {
    None,
    ContainsSomeTags(Vec<String>),
    ContainsAllTags(Vec<String>)
}
pub async fn query_images(tag_filter: TagFilter, db: &DatabaseConnection) -> Result<Vec<ImageResult>, ServerError> {
    let images_with_tags: Vec<(image::Model, Vec<tag::Model>)> = match tag_filter {
        TagFilter::None => {
            // Simplest case: select all images and join them
            // with their tags
            Image::find().find_with_related(Tag)
                .all(db).await?
        },
        TagFilter::ContainsSomeTags(tags) => {
            // Slightly more complicated: filter the images
            // to only the ones that have at least one of the tags
            Image::find().find_with_related(Tag)
                .filter(tag::Column::Name.is_in(tags))
                .all(db).await?
        },
        TagFilter::ContainsAllTags(tags) => {
            // To select the images that have all the tags,
            // we first select all the images that have some of the
            // tags, and then count how many of those tags they have
            // i.e.
            //   SELECT image.id FROM image
            //   JOIN image_tag ON image
            //   JOIN image_tag ON image.id = image_tag.image_id
            //   JOIN tag ON image_tag.tag_id = tag.id
            //   WHERE tag.name IN ('cat','dog') 
            // (replacing ('cat','dog') with our vector of tags)
            // Then, we filter the images that have the same count
            // as the number of tags (and hence has all of the provided
            // tags).
            // i.e.
            //   GROUP BY image.id
            //   HAVING COUNT(*) >= 2
            // (2 for 'cat' and 'dog', but we'd replace this with the
            // length of the vector of tags)
            todo!()
        },
    };

    let result_images: Vec<ImageResult> = images_with_tags
        .iter()
        .map(|(image, tags)| {
            // Extract names from Tags (shadowing old value)
            let tags: Vec<String> = tags.iter().map(|tag| tag.name.clone()).collect();
            ImageResult {
                url: image.url.clone(),
                label: image.label.clone(),
                tags,
            }
        })
        .collect();

    Ok(result_images)
}
