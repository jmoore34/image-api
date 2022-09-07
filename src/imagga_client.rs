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
                    match response.result {
                        Some(result) => {
                            let tags = map_result_to_tags(result);
                            println!("{}", tags.join(", "))
                        },
                        None => todo!(),
                    }

                },
                Err(err) => {
                    println!("Err: {err}")
                },
            }
        },
        Err(Error::Status(code, response)) => {
            match response.into_json::<ImaggaResponse>() {
                Ok(response) => {
                    println!("Imagga error {code}: {}", response.status.error_text)
                },
                Err(err) => {
                    println!("Error {code} while fetching Imagga error message: {}", err)
                },
            }
        }, Err(err) => {
            print!("other error: {err}")
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