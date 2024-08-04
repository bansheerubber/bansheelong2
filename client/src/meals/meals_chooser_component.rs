use chrono::NaiveDate;
use iced::{
	widget::{button, column, container, image, row, text},
	Alignment, Border, Element, Length, Padding, Shadow, Task, Theme,
};
use meals_database::{Database, MealInfo, MealPlan};
use std::{
	collections::{HashMap, HashSet},
	rc::Rc,
};
use uuid::Uuid;

use crate::{
	scrollable_menu::ScrollableMenu,
	styles::{green_button, primary_button, success_button},
	Message,
};

use super::{meal_contents, CalendarState, MealsMessage};

pub struct MealsChooser {
	current_date: NaiveDate,
	images: HashMap<String, image::Handle>,
	meals_database: Rc<Database<MealPlan>>,
	pub menu: ScrollableMenu,
	opened_meals: HashSet<Uuid>,
}

impl MealsChooser {
	pub fn new(meals_database: Rc<Database<MealPlan>>) -> (Self, Task<Message>) {
		let (menu, task) = ScrollableMenu::new();
		(
			Self {
				current_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
				images: HashMap::new(),
				meals_database,
				menu,
				opened_meals: HashSet::new(),
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
			MealsMessage::ToggleOpenMealInChooser { id } => {
				if self.opened_meals.contains(&id) {
					self.opened_meals.remove(&id);
				} else {
					self.opened_meals.insert(id);
				}

				let meal_plan = self.meals_database.get();
				let meal = meal_plan.all_meals.get(&id).unwrap();
				let url = meal.image.clone();

				Task::done(Message::FetchImage { meal_id: id, url })
			}
			MealsMessage::Scrollable(message) => self.menu.update(message),
			_ => unreachable!(),
		}
	}

	fn view_meal(&self, meal_info: &MealInfo) -> Element<MealsMessage> {
		if !self.opened_meals.contains(&meal_info.id) {
			return button(
				row![text!("{}", meal_info.name)]
					.spacing(10)
					.width(Length::Fill),
			)
			.on_press(MealsMessage::ToggleOpenMealInChooser { id: meal_info.id })
			.padding(10)
			.style(|theme: &Theme, _status| button::Style {
				background: Some(theme.extended_palette().background.strong.color.into()),
				text_color: theme.palette().text,
				border: Border::default(),
				shadow: Shadow::default(),
			})
			.into();
		}

		let mut meal_contents = meal_contents(meal_info, self.images.get(&meal_info.image), None);
		meal_contents = meal_contents.push(
			row![
				button(text!("Close"))
					.on_press(MealsMessage::ToggleOpenMealInChooser { id: meal_info.id })
					.style(|theme, _status| primary_button(theme)),
				container(text!("")).width(Length::Fill),
				button(text!("Choose"))
					.on_press(MealsMessage::SelectMealForDate {
						date: self.current_date,
						id: meal_info.id,
					})
					.style(|theme, _status| success_button(theme))
			]
			.spacing(25)
			.width(Length::Fill)
			.padding(Padding::default().top(5)),
		);

		container(meal_contents)
			.width(Length::Fill)
			.padding(10)
			.style(|theme: &Theme| theme.extended_palette().background.strong.color.into())
			.into()
	}

	pub fn view(&self) -> Element<MealsMessage> {
		let mut meals_list = column![].spacing(10);

		let meal_plan = self.meals_database.get();
		for meal_info in meal_plan.all_meals.values() {
			meals_list = meals_list.push(self.view_meal(meal_info));
		}

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
				container(text!("Random Meals"))
					.width(Length::Fill)
					.align_x(Alignment::Center)
					.align_y(Alignment::Center),
			)
			.on_press(MealsMessage::SetCalendarState(
				CalendarState::RandomChooser {
					date: self.current_date,
				},
			))
			.style(|theme, _style| green_button(theme))
			.width(Length::Fill)
			.into(),
		];

		container(
			self.menu
				.view(column![meals_list].spacing(10).into(), buttons),
		)
		.width(Length::Fill)
		.height(Length::Fill)
		.into()
	}
}
