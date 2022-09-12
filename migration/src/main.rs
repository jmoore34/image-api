use sea_orm_migration::prelude::*;

#[async_std::main]

// This allows us to run migrations manually through a binary if needed.
// However, we can typically let our server do that by itself (see lib.rs)
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
