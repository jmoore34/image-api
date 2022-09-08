//! SeaORM Entity. Generated by sea-orm-codegen 0.9.2

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "image")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub label: String,
    pub url: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::image_tag::Entity")]
    ImageTag,
}

impl Related<super::image_tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ImageTag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
