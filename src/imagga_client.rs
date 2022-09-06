use std::env::var;

use serde::Serialize;
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
            print!("{}", response.into_string().expect("convert to string"))
        },
        Err(Error::Status(code, text)) => {
            print!("error {code} : {text:#?}")
        }, Err(err) => {
            print!("other error: {err}")
        }
    }

}