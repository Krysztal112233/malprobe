use axum::extract::FromRef;
use sea_orm::DatabaseConnection;

#[derive(Debug, Clone)]
pub struct AppState {
    pub database: DatabaseConnection,
}

impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(input: &AppState) -> Self {
        input.database.clone()
    }
}
