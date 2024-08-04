use chrono::NaiveDate;
use iced::{
	widget::{button, column, container, image, row, text},
	Alignment, Element, Length, Task, Theme,
};
use meals_database::{Database, MealInfo, MealPlan};
use rand::seq::IteratorRandom;
use std::{collections::HashMap, rc::Rc};
use uuid::Uuid;

use crate::{
	scrollable_menu::ScrollableMenu,
	styles::{green_button, primary_button, success_button},
	Message,
};

use super::{meal_contents, CalendarState, MealsMessage};

pub struct RandomMealChooser {
	current_date: NaiveDate,
	current_meal_id: Uuid,
	images: HashMap<String, image::Handle>,
	meals_database: Rc<Database<MealPlan>>,
	pub menu: ScrollableMenu,
}

impl RandomMealChooser {
	pub fn new(meals_database: Rc<Database<MealPlan>>) -> (Self, Task<Message>) {
		let (menu, task) = ScrollableMenu::new();
		(
			Self {
				current_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
				current_meal_id: Uuid::new_v4(),
				images: HashMap::new(),
				meals_database,
				menu,
			},
			task,
		)
	}

	pub fn set_current_date(&mut self, date: NaiveDate) {
		self.current_date = date;
	}

	pub fn update(&mut self, event: MealsMessage) -> Task<Message> {
		match event {
			MealsMessage::FailedImage { .. } => Task::none(),
			MealsMessage::Image { bytes, url } => {
				self.images.insert(url, image::Handle::from_bytes(bytes));
				Task::none()
			}
			MealsMessage::RandomizeMeal => {
				let meal_plan = self.meals_database.get();
				let meal = meal_plan
					.all_meals
					.values()
					.choose(&mut rand::thread_rng())
					.unwrap();

				let url = meal.image.clone();
				self.current_meal_id = meal.id;

				Task::done(Message::FetchImage { url })
			}
			MealsMessage::Scrollable(message) => self.menu.update(message),
			_ => unreachable!(),
		}
	}

	fn view_meal(&self, meal_info: &MealInfo) -> Element<MealsMessage> {
		let meal_contents = meal_contents(meal_info, self.images.get(&meal_info.image));
		container(meal_contents)
			.width(Length::Fill)
			.padding(10)
			.style(|theme: &Theme| theme.extended_palette().background.strong.color.into())
			.into()
	}

	pub fn view(&self) -> Element<MealsMessage> {
		let meal_plan = self.meals_database.get();

		let buttons = vec![
			button(
				container(text!("Back"))
					.width(Length::Fill)
					.align_x(Alignment::Center)
					.align_y(Alignment::Center),
			)
			.on_press(MealsMessage::SetCalendarState(CalendarState::Calendar))
			.style(|theme, _style| primary_button(theme))
			.width(Length::Fill)
			.into(),
			button(
				container(text!("All Meals"))
					.width(Length::Fill)
					.align_x(Alignment::Center)
					.align_y(Alignment::Center),
			)
			.on_press(MealsMessage::SetCalendarState(CalendarState::Chooser {
				date: self.current_date,
			}))
			.style(|theme, _style| green_button(theme))
			.width(Length::Fill)
			.into(),
		];

		let current_meal_id = if meal_plan.all_meals.contains_key(&self.current_meal_id) {
			&self.current_meal_id
		} else {
			meal_plan.all_meals.keys().next().unwrap()
		};

		let meal_info = meal_plan.all_meals.get(current_meal_id).unwrap();

		container(
			self.menu.view(
				column![
					row![
						button(text!("Next").center())
							.on_press(MealsMessage::RandomizeMeal)
							.width(Length::Fill)
							.style(|theme, _style| primary_button(theme)),
						button(text!("Pick").center())
							.on_press(MealsMessage::SelectMealForDate {
								date: self.current_date,
								id: meal_plan.all_meals.get(current_meal_id).unwrap().id,
							})
							.width(Length::Fill)
							.style(|theme, _style| success_button(theme)),
					]
					.spacing(5),
					self.view_meal(meal_info)
				]
				.spacing(10)
				.into(),
				buttons,
			),
		)
		.width(Length::Fill)
		.height(Length::Fill)
		.into()
	}
}
