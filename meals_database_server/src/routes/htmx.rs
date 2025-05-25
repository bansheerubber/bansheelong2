use std::process::Command;

use maud::{html, Markup};
use meals_database::{
	normalize_recipe_json, Amount, Ingredient, MealInfo, RecipeJSON, RecipeStep, Units,
};
use rocket::{
	form::Form,
	get,
	http::{Cookie, CookieJar},
	post,
	response::{content::RawCss, Redirect},
	FromForm, State,
};
use uuid::Uuid;

use crate::{
	auth::User,
	rendering::{
		add_button, render_add_recipe, render_checkbox, render_ingredient, render_meal,
		render_parsed_recipe, render_shopping_list, render_step, root,
	},
	Context, Result,
};

#[get("/")]
pub async fn get_root(context: &State<Context>, _user: User) -> Result<Markup> {
	let meal_plan = context.meals_database.read().await;
	let meal_plan = meal_plan.get();

	let shopping_lists = &meal_plan.shopping_list;
	let planned_meals = &meal_plan.planned_meals;
	let all_meals = &meal_plan.all_meals;

	let shopping_list_markup = if shopping_lists.len() == 0 {
		html! {
			div class="flex flex-col items-center gap-4 px-4" {
				div class="shopping-list flex gap-2 text-lg w-full sm:w-[500px] p-2 justify-center" {
					"No shopping lists available"
				}
			}
		}
	} else {
		html! {
			@for (shopping_list_index, shopping_list) in shopping_lists.iter().enumerate() {
				(render_shopping_list(shopping_list_index, &shopping_list))
			}
		}
	};

	let mut meals = vec![];
	for meals_for_day in planned_meals.values() {
		for meal_stub in meals_for_day.iter() {
			if meal_stub.leftovers {
				continue;
			}

			meals.push((meal_stub.date, all_meals.get(&meal_stub.id).unwrap()));
		}
	}

	meals.sort_by(|(date1, _), (date2, _)| date1.cmp(date2));

	Ok(root(html! {
		div class="flex flex-col items-center gap-4 pt-6 px-4" {
			a href="/add-recipe" {
				"Add recipe"
			}

			(shopping_list_markup)

			@for meal in meals.iter() {
				(render_meal(meal.1))
			}
		}
	}))
}

#[get("/add-recipe")]
pub async fn get_add_recipe(_user: User) -> Result<Markup> {
	Ok(root(html! {
		(render_add_recipe())
	}))
}

#[get("/parsed-recipe?<url>")]
pub async fn get_parsed_recipe(url: String, _user: User) -> Result<Markup> {
	let recipe_json = Command::new("python")
		.args([
			"/home/me/bansheelong2/meals_database/src/bins/scraper.py",
			&url,
		])
		.output()
		.unwrap()
		.stdout;

	let recipe_json: RecipeJSON =
		serde_json::from_str(&String::from_utf8(recipe_json).unwrap()).unwrap();

	let normalized_recipe = normalize_recipe_json(recipe_json);

	Ok(root(render_parsed_recipe(
		&normalized_recipe.name,
		"",
		&normalized_recipe.image,
		normalized_recipe
			.ingredients
			.iter()
			.map(|ingredient| ingredient.name.clone())
			.collect::<Vec<_>>(),
		normalized_recipe
			.ingredients
			.iter()
			.map(|ingredient| ingredient.amount.value.to_string())
			.collect::<Vec<_>>(),
		normalized_recipe
			.ingredients
			.iter()
			.map(|ingredient| ingredient.amount.units.to_string())
			.collect::<Vec<_>>(),
		normalized_recipe
			.recipe
			.iter()
			.map(|ingredient| ingredient.description.clone())
			.collect::<Vec<_>>(),
	)))
}

#[derive(Debug, FromForm)]
pub struct AddMealData {
	name: String,
	serving_size: String,
	image: String,
	ingredient_amount: Vec<String>,
	ingredient_name: Vec<String>,
	ingredient_unit: Vec<String>,
	step: Vec<String>,
}

impl AddMealData {
	pub fn to_meal_info(&self) -> Result<MealInfo> {
		let mut ingredients = vec![];
		let ingredients_amount = self
			.ingredient_amount
			.len()
			.min(self.ingredient_name.len())
			.min(self.ingredient_unit.len());

		for i in 0..ingredients_amount {
			ingredients.push(Ingredient {
				amount: Amount {
					units: Units::try_from(self.ingredient_unit[i].as_str()).unwrap(),
					value: self.ingredient_amount[i].parse::<f32>().unwrap(),
				},
				name: self.ingredient_name[i].clone(),
			});
		}

		let mut recipe = vec![];
		for step in self.step.iter() {
			recipe.push(RecipeStep {
				description: step.clone(),
			});
		}

		Ok(MealInfo {
			id: Uuid::new_v4(),
			image: self.image.clone(),
			ingredients,
			name: self.name.to_string(),
			recipe,
			serving_size: self.serving_size.parse::<usize>().unwrap(),
		})
	}
}

#[post("/add-meal", data = "<data>")]
pub async fn post_add_meal(
	context: &State<Context>,
	data: Form<AddMealData>,
	_user: User,
) -> Result<Redirect> {
	let meals_database = context.meals_database.write().await;
	let mut meal_plan = meals_database.get_mut();

	let meal_info = data.to_meal_info()?;
	meal_plan.all_meals.insert(meal_info.id, meal_info);
	drop(meal_plan);

	meals_database.save();

	Ok(Redirect::to("/"))
}

#[get("/style.css")]
pub fn get_style() -> RawCss<String> {
	RawCss(std::fs::read_to_string("./meals_database_server/output.css").unwrap())
}

#[derive(Debug, FromForm)]
pub struct CheckboxState {
	checked: bool,
	name: String,
	shopping_list_index: usize,
}

#[post("/update-checkbox", data = "<checkbox>")]
pub async fn post_checkbox(
	context: &State<Context>,
	checkbox: Form<CheckboxState>,
	_user: User,
) -> Result<Markup> {
	let database = context.meals_database.write().await;
	let mut meal_plan = database.get_mut();

	let shopping_list = meal_plan
		.shopping_list
		.get_mut(checkbox.shopping_list_index)
		.unwrap();

	let shopping_list_item = shopping_list
		.items
		.iter_mut()
		.find(|item| item.name == checkbox.name)
		.unwrap();

	shopping_list_item.have = checkbox.checked;

	let shopping_list_item = shopping_list_item.clone();

	drop(meal_plan);

	database.save();

	Ok(html! { (render_checkbox(checkbox.shopping_list_index, &shopping_list_item)) })
}

#[derive(Debug, FromForm)]
pub struct AddThing {
	id: usize,
}

#[post("/add-ingredient", data = "<data>")]
pub async fn post_add_ingredient(data: Form<AddThing>, _user: User) -> Result<Markup> {
	Ok(html! {
		(render_ingredient(data.id, "", "", ""))
		(add_button("/add-ingredient", "ingredient", data.id + 1))
	})
}

#[post("/add-step", data = "<data>")]
pub async fn post_add_step(data: Form<AddThing>, _user: User) -> Result<Markup> {
	Ok(html! {
		(render_step(data.id, ""))
		(add_button("/add-step", "step", data.id + 1))
	})
}

#[get("/login")]
pub fn get_login() -> Markup {
	root(html! {
		div class="flex justify-center w-full mt-4 p-6" {
			form
				class="flex flex-col gap-2 w-full sm:w-[500px]"
				action="/login"
				method="post"
			{
				input name="username" type="text" placeholder="username";
				input name="password" type="password" placeholder="password";
				button class="submit-button" type="submit" { "Submit" }
			}
		}
	})
}

#[derive(Debug, FromForm)]
pub struct LoginForm {
	username: String,
	password: String,
}

#[post("/login", data = "<login>")]
pub async fn post_login(
	context: &State<Context>,
	login: Form<LoginForm>,
	cookies: &CookieJar<'_>,
) -> Redirect {
	let password = std::fs::read_to_string("password").expect("Could not open password file");
	if login.username != "me" || login.password != password.trim() {
		return Redirect::to("/login");
	}

	let sid = Uuid::new_v4();
	cookies.add(Cookie::build(("SID", sid.to_string())).expires(None));

	let mut valid_sids = context.valid_sids.write().await;
	valid_sids.insert(sid.clone());

	Redirect::to("/")
}
