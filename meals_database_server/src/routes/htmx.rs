use maud::{html, Markup};
use meals_database::{MealInfo, ShoppingListInfo, ShoppingListItem};
use rocket::{form::Form, get, post, response::content::RawCss, FromForm, State};

use crate::{Context, Result};

#[get("/")]
pub async fn get_root(context: &State<Context>) -> Result<Markup> {
	let meal_plan = context.meals_database.read().await;
	let meal_plan = meal_plan.get();

	let shopping_lists = &meal_plan.shopping_list;
	let planned_meals = &meal_plan.planned_meals;
	let all_meals = &meal_plan.all_meals;

	let shopping_list_markup = if shopping_lists.len() == 0 {
		html! {
			div class="flex flex-col items-center gap-4 pt-6 px-4" {
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
			(shopping_list_markup)

			@for meal in meals.iter() {
				(render_meal(meal.1))
			}
		}
	}))
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

pub fn render_shopping_list(
	shopping_list_index: usize,
	shopping_list: &ShoppingListInfo,
) -> Markup {
	html! {
		div class="shopping-list flex flex-col gap-2 text-lg w-full sm:w-[500px] p-2" {
			@for shopping_list_item in shopping_list.items.iter() {
				(render_shopping_list_item(shopping_list_index, &shopping_list_item))
			}
		}
	}
}

pub fn render_shopping_list_item(
	shopping_list_index: usize,
	shopping_list_item: &ShoppingListItem,
) -> Markup {
	html! {
		div class="grid grid-cols-[2.5rem_auto_5rem] w-full gap-2 items-center" {
			div class="flex items-center justify-center h-full" {
				(render_checkbox(shopping_list_index, shopping_list_item))
			}
			span { (shopping_list_item.name) }
			span { (format!("{} {}", shopping_list_item.amount.value, shopping_list_item.amount.units)) }
		}
	}
}

pub fn render_checkbox(
	shopping_list_index: usize,
	shopping_list_item: &ShoppingListItem,
) -> Markup {
	html! {
		input
			.shopping-list-checkbox
			hx-trigger="change"
			hx-post="/update-checkbox"
			hx-swap="outerHTML"
			hx-vals=(format!(
				r#"{{ "shopping_list_index": {}, "name": "{}", "checked": {} }}"#,
				shopping_list_index,
				shopping_list_item.name,
				!shopping_list_item.have
			))
			checked[shopping_list_item.have]
			type="checkbox";
	}
}

pub fn render_meal(meal_info: &MealInfo) -> Markup {
	html! {
		div class="meal flex flex-col gap-2 text-lg w-full sm:w-[500px] p-3 items-start" {
			img class="w-full" src=(meal_info.image);
			span { (meal_info.name) }
			span { "Serves " (meal_info.serving_size) }
			hr;
			span { "Ingredients:" }
			@for (index, ingredient) in meal_info.ingredients.iter().enumerate() {
				div class="grid grid-cols-[2.25rem,1fr,2fr] w-full" {
					span { (index + 1) "." }
					span { (ingredient.amount.value) " " (format!("{}", ingredient.amount.units)) }
					span { (ingredient.name) }
				}
			}
			hr;
			span { "Recipe:" }
			@for (index, step) in meal_info.recipe.iter().enumerate() {
				div class="grid grid-cols-[2.25rem,auto] w-full" {
					span { (index + 1) "." }
					p { (step.description) }
				}
			}
		}
	}
}

pub fn root(body: Markup) -> Markup {
	html! {
		head {
			link href="/style.css" rel="stylesheet";
			meta name="viewport" content="width=device-width, initial-scale=1.0";
		}
		script src="https://unpkg.com/htmx.org@1.9.12" crossorigin="anonymous" {}
		body {
			(body)
		}
	}
}
