use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum Units {
	#[default]
	Count,
	Cup,
	Ounce,
	Tablespoon,
	Teaspoon,
}

impl Display for Units {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Units::Count => f.write_str(""),
			Units::Cup => f.write_str("cup"),
			Units::Ounce => f.write_str("oz"),
			Units::Tablespoon => f.write_str("tbsp"),
			Units::Teaspoon => f.write_str("tsp"),
		}
	}
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Amount {
	pub units: Units,
	pub value: f32,
}

impl Amount {
	pub fn new(amount: f32, units: Units) -> Amount {
		Amount {
			value: amount,
			units,
		}
	}
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Ingredient {
	pub amount: Amount,
	pub name: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RecipeStep {
	pub description: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MealInfo {
	pub id: Uuid,
	pub image: String,
	pub ingredients: Vec<Ingredient>,
	pub name: String,
	pub recipe: Vec<RecipeStep>,
	pub serving_size: usize,
}

impl MealInfo {
	pub fn new_stub(&self, date: NaiveDate, time: Time) -> MealStub {
		MealStub {
			date,
			id: self.id,
			leftovers: false,
			time,
		}
	}
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum Time {
	Breakfast = 0,
	Lunch = 1,
	#[default]
	Dinner = 2,
}

impl Time {
	pub fn as_usize(&self) -> usize {
		match self {
			Time::Breakfast => 0,
			Time::Lunch => 1,
			Time::Dinner => 2,
		}
	}
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct MealStub {
	pub date: NaiveDate,
	pub id: Uuid,
	#[serde(default)]
	pub leftovers: bool,
	pub time: Time,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MealPlan {
	pub all_meals: HashMap<Uuid, MealInfo>,
	pub planned_meals: HashMap<NaiveDate, Vec<MealStub>>,
}
