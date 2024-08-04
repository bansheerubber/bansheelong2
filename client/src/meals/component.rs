use bytes::Bytes;
use chrono::NaiveDate;
use iced::{widget::row, Alignment, Element, Task};
use meals_database::{Database, MealPlan, MealStub, Time};
use std::rc::Rc;
use uuid::Uuid;

use crate::{calendar::Calendar, scrollable_menu::ScrollableMenuMessage, Message};

use super::{MealsChooser, MealsList, RandomMealChooser};

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
	random_meal_chooser: RandomMealChooser,
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
	Image {
		bytes: Bytes,
		url: String,
	},
	RandomizeMeal,
	SetCalendarState(CalendarState),
	SelectMealForDate {
		date: NaiveDate,
		id: Uuid,
	},
	Scrollable(ScrollableMenuMessage),
	ToggleOpenMeal {
		date: NaiveDate,
		id: Uuid,
		time: Time,
	},
	ToggleOpenMealInChooser {
		id: Uuid,
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

		(
			Self {
				calendar: Calendar::new(meals_database.clone()),
				calendar_state: CalendarState::Calendar,
				meals_chooser,
				meals_database,
				meals_list,
				random_meal_chooser,
			},
			Task::batch([
				meals_list_task,
				meals_chooser_task,
				random_meal_chooser_task,
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
				if &self.meals_list.menu.id == message.get_id() {
					self.meals_list.update(event)
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
			MealsMessage::ToggleOpenMeal { .. } => self.meals_list.update(event),
			MealsMessage::ToggleOpenMealInChooser { .. } => self.meals_chooser.update(event),
		}
	}

	pub fn view(&self) -> Element<MealsMessage> {
		row!(
			self.meals_list.view(),
			match self.calendar_state {
				CalendarState::Calendar => self.calendar.view(),
				CalendarState::Chooser { .. } => self.meals_chooser.view(),
				CalendarState::RandomChooser { .. } => self.random_meal_chooser.view(),
			}
		)
		.spacing(740 - self.calendar.width() - self.meals_list.width())
		.align_y(Alignment::Center)
		.width(740)
		.into()
	}
}
