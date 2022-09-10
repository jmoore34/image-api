use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Image::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Image::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key()
                    )
                    .col(ColumnDef::new(Image::Label).string().not_null())
                    .col(ColumnDef::new(Image::Url).string().not_null())
                    .to_owned()
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Tag::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key()
                    )
                    .col(ColumnDef::new(Tag::Name).string().not_null())
                    .to_owned()
            )
            .await?;    

        manager
            .create_table(
                Table::create()
                    .table(ImageTag::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ImageTag::ImageId)
                            .integer()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(ImageTag::TagId)
                            .integer()
                            .not_null()
                    )
                    .primary_key(
                        Index::create()
                            .col(ImageTag::ImageId)
                            .col(ImageTag::TagId)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_ImageTag_ImageId")
                            .from(ImageTag::Table, ImageTag::ImageId)
                            .to(Image::Table, Image::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_ImageTag_TagId")
                            .from(ImageTag::Table, ImageTag::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade)
                    )
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ImageTag::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Image::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Image {
    Table,
    Id,
    Label,
    Url
}

#[derive(Iden)]
pub enum Tag {
    Table,
    Id,
    Name
}

#[derive(Iden)]
pub enum ImageTag {
    Table,
    ImageId,
    TagId
}