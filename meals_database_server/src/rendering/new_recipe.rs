use maud::{html, Markup};

pub fn render_parsed_recipe(
	name: &str,
	serving_size: &str,
	image: &str,
	ingredient_names: Vec<String>,
	ingredient_amounts: Vec<String>,
	ingredient_units: Vec<String>,
	steps: Vec<String>,
) -> Markup {
	let ingredient_count = ingredient_names
		.len()
		.min(ingredient_amounts.len())
		.min(ingredient_units.len());

	html! {
		form action="/add-meal" class="flex flex-col items-center gap-4 text-xl pt-10" method="post" {
			(render_name(name, serving_size))
			(render_image(image))
			div class="flex flex-col gap-2" {
				span { "Ingredients:" }
				@for i in 0..ingredient_count {
					(render_ingredient(i, &ingredient_names[i], &ingredient_amounts[i], &ingredient_units[i]))
				}
				(add_button("/add-ingredient", "ingredient", ingredient_count))
			}
			div class="flex flex-col gap-2" {
				span { "Steps:" }
				@for (i, step) in steps.iter().enumerate() {
					(render_step(i, step))
				}
				(add_button("/add-step", "step", steps.len()))
			}
			button type="submit" {
				"Add new recipe"
			}
		}
	}
}

pub fn render_add_recipe() -> Markup {
	html! {
		form action="/parsed-recipe" class="flex text-xl justify-center gap-2 pt-10" method="get" {
			input id="url" name="url" type="text";
			button {
				"Add from URL"
			}
		}
		form action="/add-meal" class="flex flex-col items-center gap-4 text-xl pt-10" method="post" {
			(render_name("", ""))
			(render_image(""))
			div class="flex flex-col gap-2" {
				span { "Ingredients:" }
				(render_ingredient(0, "", "", ""))
				(add_button("/add-ingredient", "ingredient", 1))
			}
			div class="flex flex-col gap-2" {
				span { "Steps:" }
				(render_step(0, ""))
				(add_button("/add-step", "step", 1))
			}
			button type="submit" {
				"Add new recipe"
			}
		}
	}
}

pub fn add_button(route: &str, ty: &str, next_id: usize) -> Markup {
	html! {
		button
			hx-post=(route)
			hx-swap="outerHTML"
			hx-vals=(format!(
				r#"{{ "id": {} }}"#,
				next_id,
			))
		{
			(format!("Add {}", ty))
		}
	}
}

pub fn render_name(value: &str, serving_size: &str) -> Markup {
	html! {
		div class="flex gap-2 items-center" {
			label for="name" {
				"Name:"
			}
			input id="name" name="name" type="text" value=(value);
			input class="w-[50px]" id="serving_size" name="serving_size" type="text" value=(serving_size);
		}
	}
}

pub fn render_image(value: &str) -> Markup {
	html! {
		div class="flex gap-2 items-center" {
			label for="image" {
				"Image:"
			}
			input id="image" name="image" type="text" value=(value);
		}
	}
}

pub fn render_ingredient(number: usize, name: &str, amount: &str, unit: &str) -> Markup {
	let name_id = format!("ingredient_name[{}]", number);
	let amount_id = format!("ingredient_amount[{}]", number);
	let select_id = format!("ingredient_unit[{}]", number);

	html! {
		div class="flex gap-2 items-center" {
			label for=(name_id) {
				(format!("{}.", number + 1))
			}
			input id=(name_id) name=(name_id) type="text" value=(name);
			input class="w-[70px]" id=(amount_id) name=(amount_id) type="text" value=(amount);
			select class="h-[36px]" id=(select_id) name=(select_id) value=(unit) {
				option value="unit" {
					"Count"
				}
				option value="cup" {
					"Cup"
				}
				option value="ounce" {
					"Ounce"
				}
				option value="tablespoon" {
					"Tablespoon"
				}
				option value="teaspoon" {
					"Teaspoon"
				}
				option value="milliliters" {
					"Milliliters"
				}
			}
		}
	}
}

pub fn render_step(number: usize, value: &str) -> Markup {
	let id = format!("step[{}]", number);

	html! {
		div class="flex gap-2" {
			label class="text-xl border-none" for=(id) {
				(format!("{}.", number + 1))
			}
			textarea class="w-[400px] h-[200px] resize-none" id=(id) name=(id) {
				(value)
			}
		}
	}
}
