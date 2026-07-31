use sea_orm_migration::{prelude::*, schema::*, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(FileStatus::Type)
                    .values([
                        FileStatus::Pending,
                        FileStatus::Scanning,
                        FileStatus::Completed,
                        FileStatus::Failed,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(FileVerdict::Type)
                    .values([
                        FileVerdict::Clean,
                        FileVerdict::Suspicious,
                        FileVerdict::Malicious,
                        FileVerdict::Error,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Files::Table)
                    .if_not_exists()
                    .col(pk_uuid(Files::Id))
                    .col(string(Files::Sha256).not_null().unique_key())
                    .col(big_integer(Files::Size).not_null())
                    .col(string_null(Files::MimeType))
                    .col(string(Files::StoragePath).not_null())
                    .col(
                        ColumnDef::new(Files::Status)
                            .enumeration(
                                FileStatus::Type,
                                [
                                    FileStatus::Pending,
                                    FileStatus::Scanning,
                                    FileStatus::Completed,
                                    FileStatus::Failed,
                                ],
                            )
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(Files::Verdict)
                            .enumeration(
                                FileVerdict::Type,
                                [
                                    FileVerdict::Clean,
                                    FileVerdict::Suspicious,
                                    FileVerdict::Malicious,
                                    FileVerdict::Error,
                                ],
                            )
                            .null(),
                    )
                    .col(string_null(Files::MalwareName))
                    .col(json_binary_null(Files::Details))
                    .col(text_null(Files::Error))
                    .col(
                        timestamp_with_time_zone(Files::CreatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Files::UpdatedAt)
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone_null(Files::ScannedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_files_status")
                    .table(Files::Table)
                    .col(Files::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Files::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().if_exists().name(FileVerdict::Type).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().if_exists().name(FileStatus::Type).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Files {
    Table,
    Id,
    Sha256,
    Size,
    MimeType,
    StoragePath,
    Status,
    Verdict,
    MalwareName,
    Details,
    Error,
    CreatedAt,
    UpdatedAt,
    ScannedAt,
}

#[derive(DeriveIden)]
enum FileStatus {
    #[sea_orm(iden = "file_status")]
    Type,
    Pending,
    Scanning,
    Completed,
    Failed,
}

#[derive(DeriveIden)]
enum FileVerdict {
    #[sea_orm(iden = "file_verdict")]
    Type,
    Clean,
    Suspicious,
    Malicious,
    Error,
}
