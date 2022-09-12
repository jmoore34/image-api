// Here we export our identifiers. Normally, we don't need to use these
// when interacting with SeaORM, but they are used when interacting with
// the lower-level SeaQuery query builder (e.g. for serving a request like
// `GET /images?objects=cat,dog` where we need more advanced joins.
pub use m20220101_000001_create_table::{Image, Tag, ImageTag};
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;

// We export this so our server can run migrations on startup if they have
// not already been run. This makes deployment easier. SeaORM itself manages
// a table in the database specifically to keep track of which migrations
// have been run.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_create_table::Migration)]
    }
}
