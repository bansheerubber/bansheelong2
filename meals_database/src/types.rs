use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};
use uuid::Uuid;

#[derive(Clone, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Units {
	#[default]
	Count = 1,
	Cup = 2,
	Ounce = 0,
	Tablespoon = 3,
	Teaspoon = 4,
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

impl Units {
	pub fn is_compatible(&self, other: &Units) -> bool {
		if self == other {
			return true;
		}

		match self {
			Units::Count => false,
			Units::Cup => other.is_volume(),
			Units::Ounce => false,
			Units::Tablespoon => other.is_volume(),
			Units::Teaspoon => other.is_volume(),
		}
	}

	pub fn is_volume(&self) -> bool {
		match self {
			Units::Count => return false,
			Units::Cup => return true,
			Units::Ounce => return false,
			Units::Tablespoon => return true,
			Units::Teaspoon => return true,
		}
	}

	pub fn is_bigger(&self, other: &Units) -> Option<bool> {
		if !self.is_compatible(other) {
			return None;
		}

		let result = match self {
			Units::Count => false,
			Units::Cup => match other {
				Units::Tablespoon | Units::Teaspoon => true,
				_ => false,
			},
			Units::Ounce => false,
			Units::Tablespoon => match other {
				Units::Teaspoon => true,
				_ => false,
			},
			Units::Teaspoon => false,
		};

		Some(result)
	}

	pub fn conversion_factor(&self, other: &Units) -> Option<f32> {
		if !self.is_compatible(other) {
			return None;
		}

		if self == other {
			return Some(1.0);
		}

		let left = if self.is_bigger(other).unwrap() {
			self
		} else {
			other
		};

		let right = if self.is_bigger(other).unwrap() {
			other
		} else {
			self
		};

		let conversion = match left {
			Units::Count => unreachable!(),
			Units::Cup => match right {
				Units::Tablespoon => 16.0,
				Units::Teaspoon => 48.0,
				_ => unreachable!(),
			},
			Units::Ounce => unreachable!(),
			Units::Tablespoon => match right {
				Units::Teaspoon => 3.0,
				_ => unreachable!(),
			},
			Units::Teaspoon => unreachable!(),
		};

		if self.is_bigger(other).unwrap() {
			Some(1.0 / conversion)
		} else {
			Some(conversion)
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

	pub fn add(&self, other: &Amount) -> Option<Amount> {
		if !self.units.is_compatible(&other.units) {
			return None;
		}

		let left = if self.units.is_bigger(&other.units).unwrap() {
			self
		} else {
			other
		};

		let right = if self.units.is_bigger(&other.units).unwrap() {
			other
		} else {
			self
		};

		Some(Amount {
			units: left.units.clone(),
			value: left.value + right.value * left.units.conversion_factor(&right.units).unwrap(),
		})
	}
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Ingredient {
	pub amount: Amount,
	pub name: String,
}

impl Ingredient {
	pub fn name(&self) -> String {
		self.name.to_lowercase()
	}
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
	pub leftovers: bool,
	pub time: Time,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ShoppingListItem {
	pub amount: Amount,
	pub have: bool,
	pub name: String,
}

impl ShoppingListItem {
	pub fn name(&self) -> String {
		self.name.to_lowercase()
	}
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ShoppingListInfo {
	#[serde(default)]
	pub for_meals: Vec<MealStub>,
	pub items: Vec<ShoppingListItem>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MealPlan {
	pub all_meals: HashMap<Uuid, MealInfo>,
	pub planned_meals: HashMap<NaiveDate, Vec<MealStub>>,
	pub shopping_list: Vec<ShoppingListInfo>,
}

impl MealPlan {
	pub fn generate_shopping_list(&self) -> ShoppingListInfo {
		let mut items: HashMap<String, ShoppingListItem> = HashMap::new();

		for meals in self.planned_meals.values() {
			for meal_stub in meals.iter() {
				if meal_stub.leftovers {
					continue;
				}

				let meal = self.all_meals.get(&meal_stub.id).unwrap();
				for ingredient in meal.ingredients.iter() {
					let item = items.entry(ingredient.name()).or_insert(ShoppingListItem {
						amount: Amount::new(0.0, ingredient.amount.units.clone()),
						have: false,
						name: ingredient.name.clone(),
					});

					if item.amount.units.is_compatible(&ingredient.amount.units) {
						item.amount = item.amount.add(&ingredient.amount).unwrap();
					}
				}
			}
		}

		let mut items = items.into_values().collect::<Vec<_>>();
		items.sort_by(|item1, item2| item1.amount.units.cmp(&item2.amount.units));

		ShoppingListInfo {
			items,
			for_meals: vec![],
		}
	}
}
