use meals_database::{Amount, Ingredient, MealInfo, MealPlan, MealPlanMessage, RecipeStep, RestDatabase, Units};
use tokio::sync::mpsc::Receiver;
use std::{io::Write, str::FromStr};
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

	let (database, _): (RestDatabase<MealPlan>, Receiver<MealPlanMessage>) =
		RestDatabase::new(
			"http://bansheestorage-alt:8001/rest/meals/all",
			"http://bansheestorage-alt:8001/rest/meals/replace",
			"ws://bansheestorage-alt:8001/ws/meals-events",
		)
		.await;

	database.load().await;

	println!("1. Enter recipe");
	println!("2. Delete recipe");
	println!("3. List recipes");

	loop {
		let option = readline();
		match option.as_str() {
			"1" => {
				let recipe = enter_recipe();
				database.get_mut().all_meals.insert(recipe.id, recipe);
				database.save().await;
				break;
			}
			_ => continue,
		}
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
