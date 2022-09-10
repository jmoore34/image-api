use photon_rs::{base64_to_image, native::save_image};

// The name of the local directory we should store uploaded files in
pub static UPLOAD_DIR: &str = "uploaded_files";
// The route (from the root) that clients should use to access uploaded files
pub static FILES_ROUTE: &str = "/files";

pub fn upload(base64str: &str, id: i32) -> String {
    let image = base64_to_image(base64str);
    let path = format!("{UPLOAD_DIR}/{id}.png");
    save_image(image, &path);
    // Returned path should include site prefix:
    format!("http://localhost:3000{FILES_ROUTE}/{id}.png")
}