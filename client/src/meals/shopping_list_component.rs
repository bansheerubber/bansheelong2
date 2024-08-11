use iced::{
	widget::{button, checkbox, column, container, row, text},
	Element, Length, Task, Theme,
};
use meals_database::{MealPlan, RestDatabase, ShoppingListInfo, ShoppingListItem};
use std::sync::Arc;

use crate::{
	styles::{checkbox_style, primary_button},
	Message,
};

use super::MealsMessage;

pub struct ShoppingList {
	meals_database: Arc<RestDatabase<MealPlan>>,
}

impl ShoppingList {
	pub fn new(meals_database: Arc<RestDatabase<MealPlan>>) -> Self {
		Self { meals_database }
	}

	fn view_shopping_list_item(
		&self,
		shopping_list_index: usize,
		item: &ShoppingListItem,
	) -> Element<MealsMessage> {
		let name = item.name.clone();
		row![
			checkbox("", item.have)
				.on_toggle(move |_toggle| MealsMessage::ToggleShoppingListItem {
					name: name.clone(),
					shopping_list_index,
				})
				.style(|_theme, status| checkbox_style(status)),
			container(text!("{}", item.name)).width(250),
			container(text!("{} {}", item.amount.value, item.amount.units))
		]
		.into()
	}

	fn view_shopping_list(
		&self,
		shopping_list: &ShoppingListInfo,
		shopping_list_index: usize,
	) -> Element<MealsMessage> {
		container(
			column![
				column(
					shopping_list
						.items
						.iter()
						.map(|item| self.view_shopping_list_item(shopping_list_index, &item)),
				)
				.width(Length::Fill),
				row![button(text!("Prune"))
					.on_press(MealsMessage::PruneShoppingList {
						shopping_list_index
					})
					.style(|theme, _status| primary_button(theme))]
			]
			.spacing(20),
		)
		.padding(10)
		.width(Length::Fill)
		.style(|theme: &Theme| theme.extended_palette().background.strong.color.into())
		.into()
	}

	pub fn update(&self, message: MealsMessage) -> Task<Message> {
		match message {
			MealsMessage::GenerateShoppingList => {
				let mut meal_plan = self.meals_database.get_mut();

				let new_shopping_list = meal_plan.generate_shopping_list();
				if let Some(new_shopping_list) = new_shopping_list {
					meal_plan.shopping_list.push(new_shopping_list);
				}

				drop(meal_plan);

				let meals_database = self.meals_database.clone();
				Task::future(async move {
					meals_database.save().await;
					Message::Noop
				})
			}
			MealsMessage::PruneShoppingList {
				shopping_list_index,
			} => {
				let mut meal_plan = self.meals_database.get_mut();

				let shopping_list = meal_plan
					.shopping_list
					.get_mut(shopping_list_index)
					.unwrap();

				shopping_list.items.retain(|item| !item.have);

				if shopping_list.items.len() == 0 {
					meal_plan.shopping_list.remove(shopping_list_index);
				}

				drop(meal_plan);

				let meals_database = self.meals_database.clone();
				Task::future(async move {
					meals_database.save().await;
					Message::Noop
				})
			}
			MealsMessage::ToggleShoppingListItem {
				name,
				shopping_list_index,
			} => {
				let mut meal_plan = self.meals_database.get_mut();

				let shopping_list_item = meal_plan
					.shopping_list
					.get_mut(shopping_list_index)
					.unwrap()
					.items
					.iter_mut()
					.find(|item| item.name == name)
					.unwrap();

				shopping_list_item.have = !shopping_list_item.have;

				drop(meal_plan);

				let meals_database = self.meals_database.clone();
				Task::future(async move {
					meals_database.save().await;
					Message::Noop
				})
			}
			_ => unreachable!(),
		}
	}

	pub fn view(&self) -> Option<Element<MealsMessage>> {
		let meal_plan = self.meals_database.get();
		if meal_plan.shopping_list.len() == 0 {
			None
		} else {
			Some(
				column(
					meal_plan
						.shopping_list
						.iter()
						.enumerate()
						.map(|(index, shopping_list)| {
							self.view_shopping_list(shopping_list, index)
						}),
				)
				.spacing(10)
				.into(),
			)
		}
	}
}
