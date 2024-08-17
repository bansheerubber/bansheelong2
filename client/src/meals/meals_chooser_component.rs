use chrono::NaiveDate;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use iced::{
	widget::{button, column, container, image, row, text, Space},
	Alignment, Border, Element, Length, Padding, Shadow, Task, Theme,
};
use meals_database::{MealInfo, MealPlan, RestDatabase};
use std::{
	collections::{HashMap, HashSet},
	sync::Arc,
};
use uuid::Uuid;

use crate::{
	pt,
	scrollable_menu::ScrollableMenu,
	styles::{
		green_button, keyboard_button, keyboard_button_focused, primary_button, success_button,
	},
	Message, ICONS,
};

use super::{meal_contents, CalendarState, MealsMessage};

pub struct MealsChooser {
	current_date: NaiveDate,
	images: HashMap<String, image::Handle>,
	meals_database: Arc<RestDatabase<MealPlan>>,
	pub menu: ScrollableMenu,
	opened_meals: HashSet<Uuid>,
	search: Option<String>,
}

impl MealsChooser {
	pub fn new(meals_database: Arc<RestDatabase<MealPlan>>) -> (Self, Task<Message>) {
		let (menu, task) = ScrollableMenu::new();
		(
			Self {
				current_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
				images: HashMap::new(),
				meals_database,
				menu,
				opened_meals: HashSet::new(),
				search: None,
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
			MealsMessage::ResetChooser => {
				self.opened_meals.clear();
				self.search = None;
				Task::none()
			}
			MealsMessage::Scrollable(message) => self.menu.update(message),
			MealsMessage::SearchMeal(character) => {
				let empty = String::new();
				let mut search = self.search.as_ref().unwrap_or(&empty).clone();

				let character = match character.as_str() {
					"Sp" => " ".to_string(),
					"Bk" => {
						search.pop();
						self.search = Some(search);
						return Task::none();
					}
					character => character.to_lowercase(),
				};

				self.search = Some(format!("{}{}", search, character));
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
			_ => unreachable!(),
		}
	}

	fn keyboard_button(&self, key: &str) -> Element<MealsMessage> {
		button(text(key.to_string()))
			.on_press(MealsMessage::SearchMeal(key.to_string()))
			.padding(2)
			.width(30)
			.height(30)
			.style(|theme, status| {
				match status {
					button::Status::Pressed => keyboard_button_focused(theme),
					_ => keyboard_button(theme),
				}
			})
			.into()
	}

	fn view_meal(&self, meal_info: &MealInfo) -> Element<MealsMessage> {
		if !self.opened_meals.contains(&meal_info.id) {
			return container(
				button(row![text!("{}", meal_info.name)].spacing(10))
					.on_press(MealsMessage::ToggleOpenMealInChooser { id: meal_info.id })
					.padding([10, 0])
					.style(|theme: &Theme, _status| button::Style {
						background: Some(theme.extended_palette().background.strong.color.into()),
						text_color: theme.palette().text,
						border: Border::default(),
						shadow: Shadow::default(),
					}),
			)
			.width(Length::Fill)
			.padding([0, 10])
			.style(|theme: &Theme| theme.extended_palette().background.strong.color.into())
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

	pub fn view(&self, show_keyboard: bool) -> Element<MealsMessage> {
		let mut meals_list = column![button(
			row![
				text("\u{e8b6}").size(pt(25)).font(ICONS),
				text(if let Some(search) = &self.search {
					search.to_string()
				} else {
					"Search".to_string()
				})
			]
			.spacing(5)
		)
		.on_press(if show_keyboard {
			MealsMessage::SetCalendarState(CalendarState::Chooser {
				date: self.current_date,
			})
		} else {
			MealsMessage::SetCalendarState(CalendarState::ChooserSearch {
				date: self.current_date,
			})
		})
		.padding(Padding::default().top(10).left(10).right(10).bottom(5))
		.width(Length::Fill)
		.style(|theme, _status| keyboard_button(theme))]
		.spacing(10);

		let meal_plan = self.meals_database.get();
		let matcher = SkimMatcherV2::default();
		for meal_info in meal_plan.all_meals.values() {
			if let Some(search) = &self.search {
				let matched = matcher.fuzzy_match(&meal_info.name.to_lowercase(), &search);
				if let None = matched {
					continue;
				}
			}

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

		let keyboard: Element<MealsMessage> = if show_keyboard {
			column![
				row![
					self.keyboard_button("Q"),
					self.keyboard_button("W"),
					self.keyboard_button("E"),
					self.keyboard_button("R"),
					self.keyboard_button("T"),
					self.keyboard_button("Y"),
					self.keyboard_button("U"),
					self.keyboard_button("I"),
					self.keyboard_button("O"),
					self.keyboard_button("P"),
				]
				.spacing(5),
				row![
					Space::new(4, 0),
					self.keyboard_button("A"),
					self.keyboard_button("S"),
					self.keyboard_button("D"),
					self.keyboard_button("F"),
					self.keyboard_button("G"),
					self.keyboard_button("H"),
					self.keyboard_button("J"),
					self.keyboard_button("K"),
					self.keyboard_button("L"),
				]
				.spacing(5),
				row![
					Space::new(21, 0),
					self.keyboard_button("Z"),
					self.keyboard_button("X"),
					self.keyboard_button("C"),
					self.keyboard_button("V"),
					self.keyboard_button("B"),
					self.keyboard_button("N"),
					self.keyboard_button("M"),
					self.keyboard_button("Sp"),
					self.keyboard_button("Bk"),
				]
				.spacing(5)
			]
			.spacing(5)
			.into()
		} else {
			Space::new(0, 0).into()
		};

		let mut chooser = column![
			self.menu
				.view(column![meals_list].spacing(10).into(), buttons, 0),
			keyboard,
		]
		.width(Length::Fill)
		.height(Length::Fill);

		if show_keyboard {
			chooser = chooser.spacing(5).padding(Padding::default().bottom(5));
		}

		return chooser.into();
	}
}
