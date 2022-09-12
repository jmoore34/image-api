use entity::image;
use entity::image_tag;
use entity::prelude::*;
use entity::tag;
use futures::future::join_all;
use migration::DbErr;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::DatabaseTransaction;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::TransactionTrait;
use sea_orm::{ActiveValue::NotSet, Set};

use crate::error::ServerError;
use crate::imagga_client::ImageInput;
use crate::upload_image::upload;

type ImageId = i32;
/// A function that accesses the database and inserts an image.
/// An image can be specified by a URL or by base64 encoding.
/// A label can be provided; otherwise, it will be generated from
/// the image's provided tags.
/// This function will also insert the tags into the database if
/// they do not already exist and link them to the image via the 
/// ImageTag junction table. A single database transaction is used
/// such that any errors will cause all database mutations to be
/// rolled back.
pub async fn execute_insert_image(
    image_input: ImageInput,
    tags: Vec<String>,
    label: Option<String>,
    db: &DatabaseConnection, // Here we use a DatabaseTransaction so if anything fails, the changes will all be rolled back
) -> Result<ImageId, ServerError> {
    // Perform everything in a transaction
    // so that if something goes wrong, all the database changes get rolled back
    let txn = db.begin().await?;
    // Get the list of tag IDs from the database
    // (creating new tags as needed)
    // 1. wait for all async queries to finish
    let tag_ids = join_all(
        tags.iter()
            .map(|tag| async { get_tag_id(tag.clone(), &txn).await }),
    )
    .await;
    // 2. We have a vector of results, which we then collect into a result of vectors.
    // We then use `?` to return an error if any of the queries failed.
    let tag_ids = tag_ids.into_iter().collect::<Result<Vec<_>, DbErr>>()?;

    // Construct and insert the image metadata
    let url = match &image_input {
        ImageInput::ImageUrl(url) => url.to_owned(),
        // If no URL is available, we give it a temporary one
        // (since we need the ID in order to include the ID in the image name)
        ImageInput::ImageBase64(_) => "temporary".to_owned(),
    };
    let new_image = create_image_model(url, &tags, label).insert(&txn).await?;
    let image_id = new_image.id;

    // Now we pair the image with the associated tags
    // in the ImageTag junction table
    let image_tags = tag_ids
        .iter()
        .map(|tag_id| image_tag::ActiveModel {
            image_id: Set(new_image.id),
            tag_id: Set(*tag_id),
        })
        .collect::<Vec<_>>();
    ImageTag::insert_many(image_tags).exec(&txn).await?;

    // Now that we have an image id, we now use it in the filename of the uploaded
    // image (if the image was specified by base64 encoding). Here we upload the image
    // and then update the Image's URL in the database.
    if let ImageInput::ImageBase64(image_base64) = image_input {
        let new_image_url = upload(&image_base64, new_image.id);

        let active_model: image::ActiveModel = new_image.into();
        let updated_model = image::ActiveModel {
            url: Set(new_image_url),
            ..active_model
        };
        updated_model.update(&txn).await?;
    }
    txn.commit().await?;

    Ok(image_id)
}

/// If a tag exists by name, return its id
/// else insert a new tag and return its id
async fn get_tag_id(name: String, db: &DatabaseTransaction) -> Result<i32, DbErr> {
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

/// A small helper function to generate a label from a list of 
/// tags by separating them with commas.
fn generate_label(tags: &Vec<String>) -> String {
    if tags.is_empty() {
        "An untagged image".to_owned()
    } else {
        let tag_list = tags.join(", ");
        format!("An image containing {tag_list}.")
    }
}


/// A small helper function to construct an Image ActiveModel, i.e.
/// a model that can be inserted into the database
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
