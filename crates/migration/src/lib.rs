pub use sea_orm_migration::prelude::*;

mod m20260731_053324_create_files_table;
mod m20260731_095839_enable_pgmq_extension;
mod m20260731_181656_alter_files_for_url_sources;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260731_053324_create_files_table::Migration),
            Box::new(m20260731_095839_enable_pgmq_extension::Migration),
            Box::new(m20260731_181656_alter_files_for_url_sources::Migration),
        ]
    }
}
