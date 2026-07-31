use sea_orm_migration::prelude::*;

pub struct Migration;

// `DeriveMigrationName` derives the name from the file stem, which is "mod" for
// directory-style migrations, so the name is implemented manually instead.
impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260731_095839_enable_pgmq_extension"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!("up.sql"))
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(include_str!("down.sql"))
            .await?;
        Ok(())
    }
}
