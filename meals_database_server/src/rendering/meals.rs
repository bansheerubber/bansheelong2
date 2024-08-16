use maud::{html, Markup};
use meals_database::{MealInfo, ShoppingListInfo, ShoppingListItem};

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
