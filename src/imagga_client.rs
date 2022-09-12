use std::env::var;

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use ureq::{get, post, Error};

use crate::error::ServerError;

/// Using the api key and secret found in environmental variables, construct the authorization.
/// This authorization should be sent in the "Authorization header"
/// We're using basic authentication, i.e. where the username and password are joined with a colon
/// and then base64-encoded.
/// Here the "username" is the API key and the "password" is the API secret.
/// This function will panic if the environment variables are missing because our Imagga authorization
/// is needed to make requests to Imagga, and it's better to correct the issue of these missing 
/// environmental variables on startup than wait until a user makes a POST request (when we may
/// no longer be monitoring the server).
pub fn get_imagga_authorization() -> String {
    match (var("IMAGGA_API_KEY"), var("IMAGGA_API_SECRET")) {
        (Ok(key), Ok(secret)) => {
            let auth = base64::encode::<String>(format!("{key}:{secret}"));
            format!("Basic {auth}")
        }
        (_, _) => {
            panic!("Missing API key/secret")
        }
    }
}

/// The structure of the body of the POST request we send to Imagga
/// (when a POST request is used instead of a GET request)
/// This will be x-www-form-urlencoded.
#[derive(Serialize)]
struct ImaggaPostRequest {
    pub image_base64: String,
}

/// This enum allows the user of this Imagga client (i.e. our webserver)
/// to specify either an image URL xor an image's base64-encoded data
#[derive(Clone)]
pub enum ImageInput {
    ImageUrl(String),
    ImageBase64(String),
}
/// Given an image (URL or base64-encoded data), use our Imagga authorization to ask
/// Imagga to detect the objects in the image. Can return a 400-class ServerError (e.g.
/// if provided a URL that points to nothing) or a 500-class ServerError (e.g. the client
/// fails to deserialize a message).
pub fn get_tags_for_image(image_input: ImageInput, imagga_authorization: String) -> Result<Vec<String>, ServerError> {
    // Send the request to Imagga (pattern matching based on the type of input)
    // and store the result (which could have been a success or a failure)
    let response = match image_input {
        ImageInput::ImageUrl(image_url) => get("https://api.imagga.com/v2/tags")
            .set("Authorization", &imagga_authorization)
            .query("image_url", &image_url)
            .call(),
        ImageInput::ImageBase64(image_base64) => {
            post("https://api.imagga.com/v2/tags")
                .set("Authorization", &imagga_authorization)
                .send_form(&[("image_base64", &image_base64)])
        }
    };

    // Exhaustively convert any errors to `ServerError`s
    let response = match response {
        Ok(response) => Ok(response),

        Err(Error::Status(error_code, response)) => {
            // Whenever we recieve a non-success error HTTP code (e.g. 400)
            // We can extract the error message and forward the error code
            let error_msg = response.into_json::<ImaggaTaggingResponse>()?.status.error_text;
            // Here we package the error code given to us by Imagga.
            // However, if Imagga gave us an invalid error code, then we
            // give a 500 error since something has gone totally wrong.
            let status_code = StatusCode::from_u16(error_code).unwrap_or(StatusCode::BAD_REQUEST);
            Err(ServerError::new(
                status_code,
                format!(
                    "Received error {error_code} from Imagga: {}",
                    error_msg
                ),
            ))
        }
        Err(err) => {
            // Here we giva a HTTP 500 error because this is a catastrophic error with no feedback
            // Which means Imagga isn't working at all (e.g. it is down) or our client isn't working
            Err(ServerError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error while making request to Imagga: {err}"),
            ))
        }
    }?; // ? operator will return from the function early with the ServerError if applicable

    // Now try to deserialize the response
    match response.into_json::<ImaggaTaggingResponse>() {
        Ok(response) => {
            // Because this a HTTP 200 result, it should have been successful.
            // Hence, we expect to see the `result` field in the JSON response.
            match response.result {
                // If all goes well, we convert the deserialized response into a list of Strings
                Some(result) => Ok(map_result_to_tags(result)),
                None => {
                    // Give a HTTP 500 error because this should not happen
                    // I.e., it would be weird to get a HTTP 200 response without a `result` field
                    Err(ServerError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Received 200 OK response from Imagga but with missing result".to_owned(),
                    ))
                }
            }
        }
        Err(err) => {
            // Give HTTP 500 error because this should not happen
            // I.e, it would be weird to get a 200 response but be unable to deserialize the message.
            // This could be caused by something like a breaking API change that we don't know about.
            Err(ServerError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "Received 200 OK response from Imagga but could not deserialize: {err}."
                ),
            ))
        }
    }
}

/// Takes the Imagga response body's result object and converts it to a more usable vector
/// of strings representing the detected objects. This also implictly discards the 
/// confidence values stored in each tag.
fn map_result_to_tags(result: ImaggaTaggingResult) -> Vec<String> {
    result
        .tags
        .iter()
        .map(|tag| tag.translations.english.to_owned())
        .collect()
}

/// The top-level schema for an Imagga response. The result field is optional
/// because it can be omitted in an unsuccessful response, and we still want to
/// be able to deserialize an unsuccessful response to get more useful error information.
#[derive(Deserialize)]
struct ImaggaTaggingResponse {
    result: Option<ImaggaTaggingResult>,
    status: ImaggaStatus,
}
/// Contains the result of a successful Imagga request, which in this case
/// only has 1 field, which is the list of tags.
#[derive(Deserialize)]
struct ImaggaTaggingResult {
    tags: Vec<ImaggaTag>,
}
/// Contians the tag as well as extra metadata we don't use (e.g. confidence).
/// Imagga supports getting translations of tags in other languages, but we're 
/// only interested in (and only request) the English translation.
#[derive(Deserialize)]
struct ImaggaTag {
    #[allow(dead_code)]
    confidence: f32,
    #[serde(rename = "tag")]
    translations: ImaggaTagTranslations,
}
/// Contains the tag name in all requested languages. We only care about
/// the English translation.
#[derive(Deserialize)]
struct ImaggaTagTranslations {
    #[serde(rename = "en")]
    english: String,
}
/// Returned in every Imagga JSON response regardless of whether the 
/// request was successful. The error_text field is just "" on successful
/// responses. Status type can be success or error
#[derive(Deserialize)]
#[allow(dead_code)] // contains unused fields
struct ImaggaStatus {
    #[serde(rename = "text")]
    error_text: String,
    #[serde(rename = "type")]
    status_type: String,
}
