use std::env::var;

use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use ureq::{post, Error, get};

use crate::error::ServerError;


pub fn get_imagga_authorization() -> String {
    match (var("IMAGGA_API_KEY"), var("IMAGGA_API_SECRET")) {
        (Ok(key), Ok(secret)) => {
            let auth = base64::encode::<String>(
                format!("{key}:{secret}")
            );
            format!("Basic {auth}")
        },
        (_,_) => {
            panic!("Missing API key/secret")
        }
    }
}

#[derive(Serialize)]
struct ImaggaRequest<'a> {
    pub image_url: &'a str
}

pub fn get_tags_for_url(url: &str) -> Result<Vec<String>, ServerError> {
    let result = get("https://api.imagga.com/v2/tags")
        .set("Authorization", &get_imagga_authorization())
        .query("image_url", url)
        .call();

    match result {
        Ok(response) => {
            match response.into_json::<ImaggaResponse>() {
                Ok(response) => {
                    match response.result {
                        Some(result) => {
                            Ok(map_result_to_tags(result))
                        },
                        None => {
                            // HTTP 500 error because this should not happen
                            Err((StatusCode::INTERNAL_SERVER_ERROR,
                            "Received 200 OK response from Imagga but with missing result".to_owned()))
                        },
                    }

                },
                Err(err) => {
                    // HTTP 500 error because this should not happen
                    Err((StatusCode::INTERNAL_SERVER_ERROR,
                        "Received 200 OK response from Imagga but could not deserialize".to_owned()))
                },
            }
        },
        Err(Error::Status(code, response)) => {
            match response.into_json::<ImaggaResponse>() {
                Ok(response) => {
                    // HTTP 400 error because it is likely the fault of the user
                    // (e.g., providing a URL to an image that does not exist)
                    Err((StatusCode::BAD_REQUEST,
                        format!("Imagga: Error {code}: {}", response.status.error_text)))
                },
                Err(err) => {
                    // HTTP 500 error because this should not happen
                    Err((StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Received error {code} from Imagga but could not deserialize")))
                },
            }
        }, Err(err) => {
            // HTTP 500 error because this should not happen
            Err((StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error while making request to Imagga: {err}")))
        }
    }
}

fn map_result_to_tags(result: ImaggaResult) -> Vec::<String> {
    result
        .tags
        .iter()
        .map(|tag| tag.translations.english.to_owned())
        .collect()
}

#[derive(Deserialize)]
struct ImaggaResponse {
    result: Option<ImaggaResult>,
    status: ImaggaStatus
}
#[derive(Deserialize)]
struct ImaggaResult {
    tags: Vec<ImaggaTag>
}
#[derive(Deserialize)]
struct ImaggaTag {
    confidence: f32,
    #[serde(rename = "tag")]
    translations: ImaggaTagTranslations
}
#[derive(Deserialize)]
struct ImaggaTagTranslations {
    #[serde(rename = "en")]
    english: String
}
#[derive(Deserialize)]
struct ImaggaStatus {
    #[serde(rename = "text")]
    error_text: String,
    #[serde(rename = "type")]
    status_type: String
}