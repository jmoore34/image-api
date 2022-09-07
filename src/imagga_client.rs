use std::env::var;

use serde::{Serialize, Deserialize};
use ureq::{post, Error, get};

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

pub fn get_tags_for_url(url: &str) {
    let result = get("https://api.imagga.com/v2/tags")
        .set("Authorization", &get_imagga_authorization())
        .query("image_url", url)
        .call();

    match result {
        Ok(response) => {
            match response.into_json::<ImaggaResponse>() {
                Ok(response) => {
                    let tags = map_response_to_tags(response);
                    println!("{}", tags.join(", "))
                },
                Err(err) => {
                    println!("Err: {err}")
                },
            }
        },
        Err(Error::Status(code, text)) => {
            print!("error {code} : {text:#?}")
        }, Err(err) => {
            print!("other error: {err}")
        }
    }
}

fn map_response_to_tags(response: ImaggaResponse) -> Vec::<String> {
    response
        .result
        .tags
        .iter()
        .map(|tag| tag.translations.english.to_owned())
        .collect()
}

#[derive(Deserialize)]
struct ImaggaResponse {
    result: ImaggaResult,
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