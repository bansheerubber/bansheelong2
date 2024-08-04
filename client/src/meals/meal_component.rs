use iced::{
	widget::{column, container, image, row, text, Column},
	Element, Length, Theme,
};
use meals_database::{Ingredient, MealInfo, RecipeStep};

use super::MealsMessage;

fn view_ingredient<'a, 'b: 'a>(
	index: usize,
	ingredient: &'a Ingredient,
) -> Element<'b, MealsMessage> {
	row![
		container(text!("{}.", index + 1)).width(30),
		container(text!(
			"{} {}",
			ingredient.amount.value,
			ingredient.amount.units
		))
		.width(90),
		container(text(ingredient.name.clone()))
	]
	.into()
}

fn view_recipe_step<'a, 'b: 'a>(
	index: usize,
	recipe_step: &'a RecipeStep,
) -> Element<'b, MealsMessage> {
	row![
		container(text!("{}.", index + 1)).width(30),
		text(recipe_step.description.clone())
	]
	.into()
}

pub fn meal_contents<'a, 'b: 'a>(
	meal_info: &'a MealInfo,
	image_handle: Option<&'a image::Handle>,
) -> Column<'b, MealsMessage> {
	let image = if let Some(handle) = image_handle {
		container(image(handle.clone()))
	} else {
		container(text(""))
			.width(Length::Fill)
			.height(200)
			.style(|theme: &Theme| theme.palette().background.into())
	};

	let ingredients = column(vec![container(text("Ingredients:")).into()])
		.extend(
			meal_info
				.ingredients
				.iter()
				.enumerate()
				.map(|(index, ingredient)| view_ingredient(index, ingredient)),
		)
		.spacing(5);

	let recipe = column(vec![container(text("Recipe:")).into()])
		.extend(
			meal_info
				.recipe
				.iter()
				.enumerate()
				.map(|(index, recipe_step)| view_recipe_step(index, recipe_step)),
		)
		.spacing(5);

	column![
		image,
		text(meal_info.name.clone()),
		text!("Serves {}", meal_info.serving_size),
		container(
			container(text!(""))
				.style(|theme: &Theme| theme.extended_palette().background.weak.color.into())
				.width(Length::Fill)
				.height(2),
		)
		.padding([5, 0]),
		ingredients,
		container(
			container(text!(""))
				.style(|theme: &Theme| theme.extended_palette().background.weak.color.into())
				.width(Length::Fill)
				.height(2),
		)
		.padding([5, 0]),
		recipe,
	]
	.spacing(6)
}
