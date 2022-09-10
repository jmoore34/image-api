use std::env::var;

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use ureq::{get, post, Error};

use crate::error::ServerError;

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

#[derive(Serialize)]
struct ImaggaPostRequest {
    pub image_base64: String,
}

#[derive(Clone)]
pub enum ImageInput {
    ImageUrl(String),
    ImageBase64(String),
}
pub fn get_tags_for_image(image_input: ImageInput) -> Result<Vec<String>, ServerError> {
    let response = match image_input {
        ImageInput::ImageUrl(image_url) => get("https://api.imagga.com/v2/tags")
            .set("Authorization", &get_imagga_authorization())
            .query("image_url", &image_url)
            .call(),
        ImageInput::ImageBase64(image_base64) => {
            post("https://api.imagga.com/v2/tags")
                .set("Authorization", &get_imagga_authorization())
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
            // HTTP 500 error because this is a catastrophic error with no feedback
            // Which means Imagga isn't working at all or our client isn't working
            Err(ServerError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error while making request to Imagga: {err}"),
            ))
        }
    }?;

    // Now try to deserialize the response
    match response.into_json::<ImaggaTaggingResponse>() {
        Ok(response) => {
            match response.result {
                Some(result) => Ok(map_result_to_tags(result)),
                None => {
                    // HTTP 500 error because this should not happen
                    Err(ServerError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Received 200 OK response from Imagga but with missing result".to_owned(),
                    ))
                }
            }
        }
        Err(err) => {
            // HTTP 500 error because this should not happen
            Err(ServerError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "Received 200 OK response from Imagga but could not deserialize: {err}."
                ),
            ))
        }
    }
}

fn map_result_to_tags(result: ImaggaTaggingResult) -> Vec<String> {
    result
        .tags
        .iter()
        .map(|tag| tag.translations.english.to_owned())
        .collect()
}

#[derive(Deserialize)]
struct ImaggaTaggingResponse {
    result: Option<ImaggaTaggingResult>,
    status: ImaggaStatus,
}
#[derive(Deserialize)]
struct ImaggaTaggingResult {
    tags: Vec<ImaggaTag>,
}
#[derive(Deserialize)]
struct ImaggaTag {
    #[allow(dead_code)]
    confidence: f32,
    #[serde(rename = "tag")]
    translations: ImaggaTagTranslations,
}
#[derive(Deserialize)]
struct ImaggaTagTranslations {
    #[serde(rename = "en")]
    english: String,
}
#[derive(Deserialize)]
#[allow(dead_code)] // contains unused fields
struct ImaggaStatus {
    #[serde(rename = "text")]
    error_text: String,
    #[serde(rename = "type")]
    status_type: String,
}
