use meals_database::{Database, MealPlan, MealPlanMessage};
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use tokio::sync::broadcast::{self, Receiver, Sender};

pub struct MealsDatabaseService {
	database: Database<MealPlan>,
	sender: Sender<MealPlanMessage>,
}

impl MealsDatabaseService {
	pub fn new() -> Self {
		let (sender, _) = broadcast::channel::<MealPlanMessage>(16);

		let mut database = Database::new("meals-database.json");
		database.load();

		MealsDatabaseService { database, sender }
	}

	pub fn replace(&mut self, new_meal_plan: MealPlan) {
		let mut meal_plan = self.database.get_mut();
		*meal_plan = new_meal_plan;
		drop(meal_plan);

		self.database.save();
		self.sender.send(MealPlanMessage::Update).unwrap();
	}

	pub fn get(&self) -> RwLockReadGuard<MealPlan> {
		self.database.get()
	}

	pub fn get_mut(&self) -> RwLockWriteGuard<MealPlan> {
		self.database.get_mut()
	}
	
	pub fn save(&self) {
		self.database.save();
		self.sender.send(MealPlanMessage::Update).unwrap();
	}

	pub fn subscribe(&self) -> Receiver<MealPlanMessage> {
		self.sender.subscribe()
	}
}
