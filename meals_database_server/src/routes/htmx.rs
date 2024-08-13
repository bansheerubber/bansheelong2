use maud::{html, Markup};
use meals_database::{ShoppingListInfo, ShoppingListItem};
use rocket::{form::Form, get, post, response::content::RawCss, FromForm, State};

use crate::{Context, Result};

#[get("/")]
pub async fn get_root(context: &State<Context>) -> Result<Markup> {
	let meal_plan = context.meals_database.read().await;
	let meal_plan = meal_plan.get();

	let shopping_lists = &meal_plan.shopping_list;

	Ok(root(html! {
		div class="flex flex-col items-center gap-4 pt-6 px-4" {
			@for (shopping_list_index, shopping_list) in shopping_lists.iter().enumerate() {
				(render_shopping_list(shopping_list_index, &shopping_list))
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
			type="checkbox"
		{}
	}
}

pub fn root(body: Markup) -> Markup {
	html! {
		head {
			link href="/style.css" rel="stylesheet" {}
			meta name="viewport" content="width=device-width, initial-scale=1.0" {}
		}
		script src="https://unpkg.com/htmx.org@1.9.12" crossorigin="anonymous" {}
		body {
			(body)
		}
	}
}
