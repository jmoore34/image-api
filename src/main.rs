use std::env;

use axum::{Router, routing::{get, post}, Extension};
use migration::{Migrator, MigratorTrait};
use routes::{post_image, get_image_by_id, get_images};
use sea_orm::Database;
use tower::ServiceBuilder;
use upload_image::{UPLOAD_DIR, FILES_ROUTE};
mod imagga_client;
mod error;
mod create_image;
mod routes;
mod query_images;
mod upload_image;


#[tokio::main]
async fn main() {

    // Database setup
    let database_url = env::var("DATABASE_URL").expect("Missing DATABASE_URL environmental variable (see README.md)");
    let database_connection = Database::connect(database_url)
        .await
        .expect("Unable to connect to database");
    Migrator::up(&database_connection, None)
        .await.unwrap();
    
    // Route and extension (i.e. for database) setup    
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/images", post(post_image))
        .route("/images", get(get_images))
        .route("/image/:image_id", get(get_image_by_id))
        .merge(axum_extra::routing::SpaRouter::new(FILES_ROUTE, UPLOAD_DIR))
        .layer(
            ServiceBuilder::new()
                .layer(Extension(database_connection))
        );

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

