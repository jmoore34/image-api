use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Display;

use axum::http::StatusCode;
use entity::image;
use entity::prelude::*;
use entity::tag;
use migration::Expr;
use migration::Query;
use sea_orm::prelude::*;
use sea_orm::sea_query::*;
use sea_orm::ColumnTrait;
use sea_orm::ConnectionTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DbBackend;
use sea_orm::EntityTrait;
use sea_orm::FromQueryResult;
use sea_orm::QueryFilter;
use sea_orm::QuerySelect;
use sea_orm::Statement;
use sea_orm::Value::Int;
use serde::Serialize;

use crate::error::ServerError;

#[derive(Serialize)]
pub struct ImageResult {
    url: String,
    tags: Vec<String>,
    label: String,
    id: i32,
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
                id: image.id,
                label: image.label,
                tags,
            })
        }
    }
}
pub enum TagFilter {
    None,
    ContainsSomeTags(Vec<String>),
    ContainsAllTags(Vec<String>),
}
pub async fn query_images(
    tag_filter: TagFilter,
    db: &DatabaseConnection,
) -> Result<Vec<ImageResult>, ServerError> {
    let images_with_tags: Vec<(image::Model, Vec<tag::Model>)> = match tag_filter {
        TagFilter::None => {
            // Simplest case: select all images and join them
            // with their tags
            Image::find().find_with_related(Tag).all(db).await?
        }
        TagFilter::ContainsSomeTags(tags) => {
            // Slightly more complicated: filter the images
            // to only the ones that have at least one of the tags
            Image::find()
                .find_with_related(Tag)
                .filter(tag::Column::Name.is_in(tags))
                .all(db)
                .await?
        }
        TagFilter::ContainsAllTags(tags) => {
            // First, we fetch the ids of all the images that have all those tags
            let image_ids = get_image_ids_that_have_all_tags(tags, db).await?;

            // Now that we have the image ids of the images with all the provided tags,
            // we can fetch all the info about those images
            Image::find()
                .find_with_related(Tag)
                .filter(image::Column::Id.is_in(image_ids))
                .all(db)
                .await?
        }
    };

    let result_images: Vec<ImageResult> = images_with_tags
        .iter()
        .map(|(image, tags)| {
            // Extract names from Tags (shadowing old value)
            let tags: Vec<String> = tags.iter().map(|tag| tag.name.clone()).collect();
            ImageResult {
                url: image.url.clone(),
                id: image.id,
                label: image.label.clone(),
                tags,
            }
        })
        .collect();

    Ok(result_images)
}

/// Fetch the ids of the images that have all the tags
/// in the provided list.
async fn get_image_ids_that_have_all_tags(
    tags: Vec<String>,
    db: &DatabaseConnection,
) -> Result<Vec<i32>, ServerError> {
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
    //   HAVING COUNT(*) = 2
    // (2 for 'cat' and 'dog', but we'd replace this with the
    // length of the vector of tags)
    let num_tags: Result<i32, _> = tags.len().try_into();
    let num_tags = match num_tags {
        Ok(num_tags) => Ok(num_tags),
        Err(_) => Err(ServerError::new(
            StatusCode::BAD_REQUEST,
            "Too many tags provided".into(),
        )),
    }?;

    let image_ids_query = Query::select()
        .column((migration::Image::Table, migration::Image::Id))
        .expr(Expr::asterisk().count())
        .from(migration::Image::Table)
        .join(
            migration::JoinType::InnerJoin,
            migration::ImageTag::Table,
            Expr::tbl(migration::Image::Table, migration::Image::Id)
                .equals(migration::ImageTag::Table, migration::ImageTag::ImageId),
        )
        .join(
            migration::JoinType::InnerJoin,
            migration::Tag::Table,
            Expr::tbl(migration::ImageTag::Table, migration::ImageTag::TagId)
                .equals(migration::Tag::Table, migration::Tag::Id),
        )
        .and_where(Expr::tbl(migration::Tag::Table, migration::Tag::Name).is_in(tags))
        .group_by_col((migration::Image::Table, migration::Image::Id))
        .and_having(Func::count(Expr::asterisk()).equals(SimpleExpr::Value(Int(Some(num_tags)))))
        .to_owned();
    // Here we actually execute the query to get all the correct image ids
    let image_ids =
        IdOnlyResult::find_by_statement(db.get_database_backend().build(&image_ids_query))
            .all(db)
            .await?;

    // Now we turn the result of the query into a more usable vector of i32s
    Ok(image_ids.iter().map(|result| result.id).collect())
}

// Struct we use to extract only the id from
// the result of a query
#[derive(Debug, FromQueryResult)]
struct IdOnlyResult {
    id: i32,
}
