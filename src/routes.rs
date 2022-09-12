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

/// This struct is deserialized from the JSON body
/// of a `POST /images` request. It specifies whether the user
/// wants object detection or not, as well as gives the user 
/// the option to specify the image's label (one will be 
/// generated automatically otherwise). 
/// The user has the option of specifying an image URL or 
/// an image's base64-encoded data; however, the user should
/// do only one of these things. A HTTP 400 error will be given
/// if the user tries to give both or neither of the `image_url`
/// and `image_base64` fields.
#[derive(Deserialize)]
pub struct NewImageRequest {
    image_url: Option<String>,
    image_base64: Option<String>,
    label: Option<String>,
    object_detection: bool,
}

/// The route handler for the `POST /images` endpoint. The JSON
/// body is deserialized into the NewImageRequest struct. A 400 or 500
/// class error can be returned depending on whether the user was at fault.
/// If no errors occur, the image is inserted into the database and the
/// resulting inserted images is serialized and sent back to the user.
/// If the insert fails mid-request, its changes to the database will
/// be rolled back (see execute_insert_image implementation.)
pub async fn post_image(
    Json(request): Json<NewImageRequest>,
    Extension(ref db): Extension<DatabaseConnection>,
    Extension(imagga_authorization): Extension<String>,
) -> Result<Json<ImageResult>, ServerError> {
    // Pattern match on the input to make sure that the user has provided an image
    // URL or base64-encoded data but not both/neither.
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

/// The route handler for the `GET /image/{imageId}` endpoint. Fetches the image and
/// returns it as JSON, unless it doesn't exist, in which case it returns a 404.
pub async fn get_image_by_id(
    Path(image_id): Path<i32>,
    Extension(ref db): Extension<DatabaseConnection>,
) -> Result<axum::Json<ImageResult>, ServerError> {
    Ok(Json(query_image_by_id(image_id, db).await?))
}

/// The query parameters for the `GET /images` endpoint.
/// `objects` is used for requesting images that contain all specified objects.
/// `some_objects` is used for requesting images that contain some of the
/// specified objects.
/// Neither query parameter is necessary, and if neither are provided, all
/// images will be returned.
/// However, passing both `objects` and `some_objects` query parameters is not
/// allowed and will result in a HTTP 400 Bad Request response.
#[derive(Deserialize)]
pub struct GetImagesQueryParams {
    objects: Option<String>, // request images containing all objects in a comma-separated list
    some_objects: Option<String> // request images containing 1+ objects in a comma separated list
}
/// The endpoint for the `GET /images` route (as well as with the `objects` and `some_objects`
/// query parameters, as per the GetImagesQueryParameters struct). Returns a JSON array of images
/// that include a list of their associated tags.
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
