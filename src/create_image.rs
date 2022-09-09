use entity::image;
use entity::image_tag;
use entity::prelude::*;
use entity::tag;
use futures::future::join_all;
use migration::DbErr;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveValue::NotSet, Set};

fn create_image_model(
    url: String,
    tags: &Vec<String>,
    label: Option<String>,
) -> image::ActiveModel {
    let label = match label {
        Some(label) => label,
        None => generate_label(&tags),
    };

    image::ActiveModel {
        id: NotSet,
        label: Set(label),
        url: Set(url),
    }
}

fn generate_label(tags: &Vec<String>) -> String {
    if tags.is_empty() {
        "An untagged image".to_owned()
    } else {
        let tag_list = tags.join(", ");
        format!("An image containing {tag_list}.")
    }
}

pub type ImageId = i32;
pub async fn execute_insert_image(
    url: String,
    tags: Vec<String>,
    label: Option<String>,
    db: &DatabaseConnection,
) -> Result<ImageId, DbErr> {
    // Get the list of tag IDs from the database
    // (creating new tags as needed)
    // 1. wait for all async queries to finish
    let tag_ids = join_all(
        tags.iter()
            .map(|tag| async { get_tag_id(tag.clone(), db).await }),
    )
    .await;
    // 2. We have a vector of results, which we then collect into a result of vectors.
    // We then use `?` to return an error if any of the queries failed.
    let tag_ids = tag_ids.into_iter().collect::<Result<Vec<_>, DbErr>>()?;

    // Construct and insert the image metadata
    let new_image = create_image_model(url, &tags, label).insert(db).await?;

    // Now we pair the image with the associated tags
    // in the ImageTag junction table
    let image_tags = tag_ids
        .iter()
        .map(|tag_id| image_tag::ActiveModel {
            image_id: Set(new_image.id),
            tag_id: Set(*tag_id),
        })
        .collect::<Vec<_>>();
    ImageTag::insert_many(image_tags).exec(db).await?;

    Ok(new_image.id)
}

// If a tag exists by name, return its id
// else insert a new tag and return its id
async fn get_tag_id(name: String, db: &DatabaseConnection) -> Result<i32, DbErr> {
    let existing = Tag::find()
        .filter(tag::Column::Name.eq(name.to_owned()))
        .one(db)
        .await?;

    match existing {
        Some(tag) => Ok(tag.id),
        None => {
            let new_tag = tag::ActiveModel {
                id: NotSet,
                name: Set(name),
            }
            .insert(db)
            .await?;

            Ok(new_tag.id)
        }
    }
}
