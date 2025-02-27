use serde::{Deserialize, Serialize};

use crate::{Amount, Ingredient, RecipeStep, Units};

#[derive(Debug, Deserialize, Serialize)]
pub struct RecipeJSON {
	author: String,
	canonical_url: String,
	category: String,
	cuisine: String,
	description: String,
	host: String,
	image: String,
	ingredients: Vec<String>,
	instructions_list: Vec<String>,
	title: String,
	yields: String,
}

pub fn parse_amount(amount: &str) -> Result<f32, ()> {
	if let Ok(amount) = amount.parse::<f32>() {
		return Ok(amount);
	}

	match amount {
		"½" => Ok(0.5),
		"⅓" => Ok(0.3),
		"⅔" => Ok(0.6),
		"¼" => Ok(0.25),
		"¾" => Ok(0.75),
		"⅕" => Ok(0.2),
		"⅖" => Ok(0.4),
		"⅗" => Ok(0.6),
		"⅘" => Ok(0.8),
		"⅙" => Ok(0.16),
		"⅚" => Ok(0.83),
		"⅐" => Ok(0.14),
		"⅛" => Ok(0.125),
		"⅜" => Ok(0.375),
		"⅝" => Ok(0.625),
		"⅞" => Ok(0.875),
		"⅑" => Ok(0.111),
		"⅒," => Ok(0.1),
		_ => Err(()),
	}
}

pub fn cleanup_recipe_step(step: &str) -> String {
	step.replace("\u{2022} ", "").trim().into()
}

pub fn normalize_recipe_json(json: RecipeJSON) -> NormalizedRecipe {
	let mut ingredients: Vec<Ingredient> = vec![];
	for ingredient in json.ingredients.iter() {
		let split = ingredient.split(" ").collect::<Vec<&str>>();
		if split.len() > 2 {
			let Ok(units): Result<Units, ()> = split[1].try_into() else {
				println!("!!! Could not parse ingredient '{}'", ingredient);
				continue;
			};

			let Ok(amount) = parse_amount(split[0]) else {
				println!("!!! Could not parse ingredient '{}'", ingredient);
				continue;
			};

			let rest = &split[2..];

			ingredients.push(Ingredient {
				amount: Amount {
					units,
					value: amount,
				},
				name: rest.join(" ").to_string(),
			});
		} else {
			println!("!!! Could not parse ingredient '{}'", ingredient);
		}
	}

	let mut recipe = vec![];
	for step in json.instructions_list.iter() {
		recipe.push(RecipeStep {
			description: cleanup_recipe_step(&step),
		});
	}

	return NormalizedRecipe {
		image: json.image,
		ingredients,
		name: json.title,
		recipe,
	};
}

#[derive(Debug)]
pub struct NormalizedRecipe {
	pub image: String,
	pub ingredients: Vec<Ingredient>,
	pub name: String,
	pub recipe: Vec<RecipeStep>,
}
