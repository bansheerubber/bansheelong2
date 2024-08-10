use meals_database::{Database, MealPlan};
use serde::Serialize;
use std::sync::RwLockReadGuard;
use tokio::sync::broadcast::{self, Receiver, Sender};

#[derive(Clone, Debug, Serialize)]
pub enum MealsDatabaseServiceMessage {
	Update,
}

pub struct MealsDatabaseService {
	database: Database<MealPlan>,
	sender: Sender<MealsDatabaseServiceMessage>,
}

impl MealsDatabaseService {
	pub fn new() -> Self {
		let (sender, _) = broadcast::channel::<MealsDatabaseServiceMessage>(16);

		let mut database = Database::new("meals-database.json");
		database.load();

		MealsDatabaseService { database, sender }
	}

	pub fn replace(&mut self, new_meal_plan: MealPlan) {
		let mut meal_plan = self.database.get_mut();
		*meal_plan = new_meal_plan;
		self.database.save();
		self.sender.send(MealsDatabaseServiceMessage::Update).unwrap();
	}

	pub fn get(&self) -> RwLockReadGuard<MealPlan> {
		self.database.get()
	}

	pub fn subscribe(&self) -> Receiver<MealsDatabaseServiceMessage> {
		self.sender.subscribe()
	}
}
