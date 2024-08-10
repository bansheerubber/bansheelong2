use std::sync::Arc;
use tokio::sync::RwLock;

use crate::services::MealsDatabaseService;

pub struct Context {
	pub meals_database: Arc<RwLock<MealsDatabaseService>>,
}

impl Context {
	pub fn new() -> Self {
		Self {
			meals_database: Arc::new(RwLock::new(MealsDatabaseService::new())),
		}
	}
}
