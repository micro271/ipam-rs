use crate::database::RepositoryInjection;
use sqlx::Postgres;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub type StateType = Arc<AppState>;

#[derive(Debug)]
pub struct AppState {
    pub db: RepositoryInjection<Postgres>,
    pub heavy_task: Semaphore,
}

impl AppState {
    pub fn new(db: RepositoryInjection<Postgres>, heavy_task: Semaphore) -> Self {
        Self { db, heavy_task }
    }
}

impl AppState {
    pub fn heavy_task(&self) -> &Semaphore {
        &self.heavy_task
    }
}

impl std::ops::Deref for AppState {
    type Target = RepositoryInjection<Postgres>;
    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl std::ops::DerefMut for AppState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}
