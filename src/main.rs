use std::env;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use imagga_client::get_imagga_authorization;
use migration::{Migrator, MigratorTrait};
use routes::{get_image_by_id, get_images, post_image};
use sea_orm::Database;
use tower::ServiceBuilder;
use upload_image::{FILES_ROUTE, UPLOAD_DIR};
mod create_image;
mod error;
mod imagga_client;
mod query_images;
mod routes;
mod upload_image;

#[tokio::main]
async fn main() {
    // Database setup
    let database_url = env::var("DATABASE_URL")
        .expect("Missing DATABASE_URL environmental variable (see README.md)");
    let database_connection = Database::connect(database_url)
        .await
        .expect("Unable to connect to database");
    Migrator::up(&database_connection, None).await.unwrap();

    let imagga_auth = get_imagga_authorization();

    // Route and extension (i.e. for database) setup
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/images", post(post_image))
        .route("/images", get(get_images))
        .route("/image/:image_id", get(get_image_by_id))
        .merge(axum_extra::routing::SpaRouter::new(FILES_ROUTE, UPLOAD_DIR))
        .layer(Extension(database_connection))
        .layer(Extension(imagga_auth));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
