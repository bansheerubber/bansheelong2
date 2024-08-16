use std::{collections::HashSet, sync::Arc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::services::MealsDatabaseService;

pub struct Context {
	pub meals_database: Arc<RwLock<MealsDatabaseService>>,
	pub valid_sids: Arc<RwLock<HashSet<Uuid>>>,
}

impl Context {
	pub fn new() -> Self {
		Self {
			meals_database: Arc::new(RwLock::new(MealsDatabaseService::new())),
			valid_sids: Arc::new(RwLock::new(HashSet::new())),
		}
	}
}
