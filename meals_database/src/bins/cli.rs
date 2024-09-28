use meals_database::{
	Amount, Ingredient, MealInfo, MealPlan, MealPlanMessage, RecipeStep, RestDatabase, Units,
};
use std::{io::Write, str::FromStr};
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

#[tokio::main]
async fn main() {
	/*env_logger::init();

	let (mut database, mut receiver): (RestDatabase<MealPlan>, Receiver<MealPlanMessage>) =
		RestDatabase::new(
			"http://0.0.0.0:8001/rest/meals/all",
			"http://0.0.0.0:8001/rest/meals/replace",
			"ws://0.0.0.0:8001/ws/meals-events",
		)
		.await;

	database.load().await;
	database.save().await;

	select! {
		message = receiver.recv().fuse() => {
			println!("{:?}", message);
		}
		_ = database.recv_loop().fuse() => {

		}
	}*/

	let (database, _): (RestDatabase<MealPlan>, Receiver<MealPlanMessage>) = RestDatabase::new(
		"http://bansheestorage:8001/rest/meals/all",
		"http://bansheestorage:8001/rest/meals/replace",
		"ws://bansheestorage:8001/ws/meals-events",
	)
	.await;

	database.load().await;

	loop {
		println!("1. Enter recipe");
		println!("2. Delete recipe");
		println!("3. Edit recipes");
		println!("4. List recipes");

		let option = readline();
		match option.as_str() {
			"1" => {
				let recipe = enter_recipe();
				database.get_mut().all_meals.insert(recipe.id, recipe);
				database.save().await;
			}
			"3" => {
				let mut meal_plan = database.get_mut();
				let mut index_to_meal_id = vec![];

				let mut meals = meal_plan
					.all_meals
					.values_mut()
					.collect::<Vec<&mut MealInfo>>();

				meals.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

				for meal in meals.iter_mut() {
					let index = index_to_meal_id.len();

					println!("#{:<5} {}", index + 1, meal.name);
					index_to_meal_id.push(meal);
				}

				let index: usize = read_number().unwrap();
				let meal = index_to_meal_id.get_mut(index - 1).unwrap();

				println!("Edit '{}':", meal.name);
				edit_meal(meal);

				drop(meal_plan);
				database.save().await;
			}
			"4" => {
				let meal_plan = database.get();
				let mut meals = meal_plan.all_meals.values().collect::<Vec<_>>();
				meals.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

				for meal in meals.iter() {
					println!("{}", meal.name);
					println!("  Image: {}", meal.image);
				}
			}
			_ => continue,
		}
	}
}

fn edit_name(meal: &mut MealInfo) {
	println!("Enter name:");
	meal.name = readline();
}

fn edit_serving_size(meal: &mut MealInfo) {
	println!("Enter serving size:");
	meal.serving_size = read_number().unwrap();
}

fn edit_image(meal: &mut MealInfo) {
	println!("Enter image url:");
	meal.image = readline();
}

fn edit_ingredients(meal: &mut MealInfo) {
	for (index, ingredient) in meal.ingredients.iter().enumerate() {
		println!(
			"#{:<5} {}, {} {}",
			index + 1,
			ingredient.name,
			ingredient.amount.value,
			ingredient.amount.units,
		);
	}

	let max = meal.ingredients.len();
	let ingredient = meal
		.ingredients
		.get_mut(read_number_in_range(1, max).unwrap() - 1)
		.unwrap();

	let units = read_units().unwrap();

	println!("Enter amount:");
	let value = read_number().unwrap();

	println!("Enter name:");
	let name = readline();

	ingredient.amount = Amount { units, value };
	ingredient.name = name;
}

fn edit_steps(meal: &mut MealInfo) {
	for (index, step) in meal.recipe.iter().enumerate() {
		println!("#{:<5} {}", index + 1, step.description);
	}

	let max = meal.recipe.len();
	let step = meal
		.recipe
		.get_mut(read_number_in_range(1, max).unwrap() - 1)
		.unwrap();

	println!("Enter step:");
	step.description = readline();
}

fn edit_meal(meal: &mut MealInfo) {
	println!("1. Edit name");
	println!("2. Edit serving size");
	println!("3. Edit image url");
	println!("4. Edit ingredients");
	println!("5. Edit steps");

	loop {
		let option = readline();
		match option.as_str() {
			"1" => edit_name(meal),
			"2" => edit_serving_size(meal),
			"3" => edit_image(meal),
			"4" => edit_ingredients(meal),
			"5" => edit_steps(meal),
			_ => continue,
		}

		break;
	}
}

fn enter_recipe() -> MealInfo {
	println!("Enter name:");
	let name = readline();

	println!("Enter serving size:");
	let Some(serving_size) = read_number() else {
		panic!();
	};

	println!("Enter image url:");
	let image = readline();

	let mut ingredients = vec![];
	loop {
		let Some(units) = read_units() else {
			break;
		};

		println!("Enter amount:");
		let Some(value) = read_number() else {
			break;
		};

		println!("Enter name:");
		let name = readline();
		if name.len() == 0 {
			break;
		}

		ingredients.push(Ingredient {
			amount: Amount { units, value },
			name,
		});
	}

	let mut recipe = vec![];
	loop {
		println!("Enter step:");
		let description = readline();
		if description.len() == 0 {
			break;
		}

		recipe.push(RecipeStep { description });
	}

	return MealInfo {
		id: Uuid::new_v4(),
		image,
		ingredients,
		name,
		recipe,
		serving_size,
	};
}

fn read_number<T>() -> Option<T>
where
	T: FromStr,
{
	loop {
		let value = readline();
		if value.len() == 0 {
			return None;
		}

		if let Ok(value) = value.parse::<T>() {
			return Some(value);
		} else {
			continue;
		}
	}
}

fn read_number_in_range<T>(min: T, max: T) -> Option<T>
where
	T: FromStr + Ord,
{
	loop {
		let value = readline();
		if value.len() == 0 {
			return None;
		}

		if let Ok(value) = value.parse::<T>() {
			if value < min || value > max {
				continue;
			}

			return Some(value);
		} else {
			continue;
		}
	}
}

fn read_units() -> Option<Units> {
	println!("Enter ingredients:");
	println!("1. Count");
	println!("2. Cups");
	println!("3. Ounces");
	println!("4. Tablespoons");
	println!("5. Teaspoons");

	loop {
		let ingredient = readline();
		match ingredient.as_str() {
			"1" => return Some(Units::Count),
			"2" => return Some(Units::Cup),
			"3" => return Some(Units::Ounce),
			"4" => return Some(Units::Tablespoon),
			"5" => return Some(Units::Teaspoon),
			"" => return None,
			_ => continue,
		}
	}
}

fn readline() -> String {
	print!("> ");
	std::io::stdout().flush().unwrap();
	let mut buffer = String::new();
	std::io::stdin().read_line(&mut buffer).unwrap();
	buffer.trim_end().into()
}
