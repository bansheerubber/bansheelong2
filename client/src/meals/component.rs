use bytes::Bytes;
use chrono::NaiveDate;
use iced::{
	widget::{column, container, row},
	Alignment, Element, Length, Task,
};
use meals_database::{Database, MealPlan, MealStub, Time};
use std::rc::Rc;
use uuid::Uuid;

use crate::{
	calendar::Calendar,
	scrollable_menu::{ScrollableMenu, ScrollableMenuMessage},
	Message,
};

use super::{shopping_list_component::ShoppingList, MealsChooser, MealsList, RandomMealChooser};

#[derive(Clone, Debug)]
pub enum CalendarState {
	Calendar,
	Chooser { date: NaiveDate },
	RandomChooser { date: NaiveDate },
}

pub struct Meals {
	calendar: Calendar,
	calendar_state: CalendarState,
	meals_chooser: MealsChooser,
	meals_database: Rc<Database<MealPlan>>,
	meals_list: MealsList,
	meals_list_menu: ScrollableMenu,
	random_meal_chooser: RandomMealChooser,
	shopping_list: ShoppingList,
}

#[derive(Clone, Debug)]
pub enum MealsMessage {
	AddMonth(isize),
	DeletePlannedMeal {
		date: NaiveDate,
		time: Time,
	},
	FailedImage {
		url: String,
	},
	GenerateShoppingList,
	Image {
		bytes: Bytes,
		url: String,
	},
	PruneShoppingList {
		shopping_list_index: usize,
	},
	RandomizeMeal,
	SetCalendarState(CalendarState),
	SelectMealForDate {
		date: NaiveDate,
		id: Uuid,
	},
	Scrollable(ScrollableMenuMessage),
	ToggleLeftovers {
		date: NaiveDate,
		time: Time,
	},
	ToggleOpenMeal {
		date: NaiveDate,
		id: Uuid,
		time: Time,
	},
	ToggleOpenMealInChooser {
		id: Uuid,
	},
	ToggleShoppingListItem {
		name: String,
		shopping_list_index: usize,
	},
}

impl From<ScrollableMenuMessage> for MealsMessage {
	fn from(message: ScrollableMenuMessage) -> Self {
		MealsMessage::Scrollable(message)
	}
}

impl Meals {
	pub fn new() -> (Self, Task<Message>) {
		let mut meals_database = Database::new("meals-database.json");
		meals_database.load();
		let meals_database = Rc::new(meals_database);

		let (meals_list, meals_list_task) = MealsList::new(meals_database.clone());
		let (meals_chooser, meals_chooser_task) = MealsChooser::new(meals_database.clone());
		let (random_meal_chooser, random_meal_chooser_task) =
			RandomMealChooser::new(meals_database.clone());

		let shopping_list = ShoppingList::new(meals_database.clone());

		let (meals_list_menu, meals_list_menu_task) = ScrollableMenu::new();

		(
			Self {
				calendar: Calendar::new(meals_database.clone()),
				calendar_state: CalendarState::Calendar,
				meals_chooser,
				meals_database,
				meals_list,
				meals_list_menu,
				random_meal_chooser,
				shopping_list,
			},
			Task::batch([
				meals_list_task,
				meals_chooser_task,
				random_meal_chooser_task,
				meals_list_menu_task,
			]),
		)
	}

	pub fn update(&mut self, event: MealsMessage) -> Task<Message> {
		match event {
			MealsMessage::AddMonth(_) => self.calendar.update(event),
			MealsMessage::DeletePlannedMeal { date, time } => {
				let mut meal_plan = self.meals_database.get_mut();
				let vec = meal_plan.planned_meals.get_mut(&date).unwrap();
				vec.retain(|meal_stub| meal_stub.time != time);

				if vec.len() == 0 {
					meal_plan.planned_meals.remove(&date);
				}

				drop(meal_plan);

				self.meals_database.save();

				Task::none()
			}
			MealsMessage::FailedImage { .. } | MealsMessage::Image { .. } => {
				self.meals_list.update(event.clone());
				self.meals_chooser.update(event.clone());
				self.random_meal_chooser.update(event);
				Task::none()
			}
			MealsMessage::RandomizeMeal => self.random_meal_chooser.update(event),
			MealsMessage::Scrollable(ref message) => {
				if &self.meals_list_menu.id == message.get_id() {
					self.meals_list_menu.update(message.clone())
				} else if &self.random_meal_chooser.menu.id == message.get_id() {
					self.random_meal_chooser.update(event)
				} else {
					self.meals_chooser.update(event)
				}
			}
			MealsMessage::SelectMealForDate { date, id: meal_id } => {
				let mut meal_plan = self.meals_database.get_mut();
				let meal_id = meal_plan.all_meals.get(&meal_id).unwrap().id;

				meal_plan
					.planned_meals
					.entry(date)
					.or_default()
					.push(MealStub {
						date,
						id: meal_id,
						leftovers: false,
						time: Time::default(),
					});

				drop(meal_plan);

				self.meals_database.save();

				Task::done(Message::Meals(MealsMessage::SetCalendarState(
					CalendarState::Calendar,
				)))
			}
			MealsMessage::SetCalendarState(state) => {
				self.calendar_state = state;

				match &self.calendar_state {
					CalendarState::Chooser { date } => {
						self.meals_chooser.set_current_date(date.clone());
						Task::done(Message::Meals(MealsMessage::Scrollable(
							ScrollableMenuMessage::Reset {
								id: self.meals_chooser.menu.id.clone(),
							},
						)))
					}
					CalendarState::RandomChooser { date } => {
						self.random_meal_chooser.set_current_date(date.clone());
						Task::batch([
							Task::done(Message::Meals(MealsMessage::Scrollable(
								ScrollableMenuMessage::Reset {
									id: self.random_meal_chooser.menu.id.clone(),
								},
							))),
							Task::done(Message::Meals(MealsMessage::RandomizeMeal)),
						])
					}
					_ => Task::none(),
				}
			}
			MealsMessage::ToggleLeftovers { date, time } => {
				let mut meal_plan = self.meals_database.get_mut();

				let meal_stub = meal_plan
					.planned_meals
					.get_mut(&date)
					.unwrap()
					.iter_mut()
					.find(|meal_stub| meal_stub.time == time)
					.unwrap();

				meal_stub.leftovers = !meal_stub.leftovers;

				drop(meal_plan);

				self.meals_database.save();

				Task::none()
			}
			MealsMessage::ToggleOpenMeal { .. } => self.meals_list.update(event),
			MealsMessage::ToggleOpenMealInChooser { .. } => self.meals_chooser.update(event),
			MealsMessage::ToggleShoppingListItem { .. }
			| MealsMessage::GenerateShoppingList
			| MealsMessage::PruneShoppingList { .. } => self.shopping_list.update(event),
		}
	}

	pub fn view(&self) -> Element<MealsMessage> {
		row!(
			container(
				self.meals_list_menu.view(
					column![self.shopping_list.view(), self.meals_list.view()]
						.spacing(10)
						.into(),
					vec![]
				)
			)
			.width(400)
			.height(Length::Fill),
			match self.calendar_state {
				CalendarState::Calendar => self.calendar.view(),
				CalendarState::Chooser { .. } => self.meals_chooser.view(),
				CalendarState::RandomChooser { .. } => self.random_meal_chooser.view(),
			}
		)
		.spacing(720 - self.calendar.width() - self.meals_list.width())
		.align_y(Alignment::Center)
		.width(720)
		.into()
	}
}
