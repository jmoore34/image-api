use std::env;

use axum::{Router, routing::get, Extension};
use imagga_client::get_tags_for_url;
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use tower::ServiceBuilder;
mod imagga_client;
mod error;


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
        .layer(
            ServiceBuilder::new()
                .layer(Extension(database_connection))
        );

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

