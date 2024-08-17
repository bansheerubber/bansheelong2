use bytes::Bytes;
use chrono::NaiveDate;
use futures::{executor::block_on, select, FutureExt, SinkExt};
use iced::{
	stream,
	widget::{button, column, container, row, text},
	Alignment, Element, Length, Task,
};
use meals_database::{MealPlan, MealPlanMessage, MealStub, RestDatabase, Time};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
	calendar::Calendar,
	scrollable_menu::{ScrollableMenu, ScrollableMenuMessage},
	styles::primary_button,
	Message,
};

use super::{shopping_list_component::ShoppingList, MealsChooser, MealsList, RandomMealChooser};

#[derive(Clone, Debug)]
pub enum CalendarState {
	Calendar,
	Chooser { date: NaiveDate },
	ChooserSearch { date: NaiveDate },
	RandomChooser { date: NaiveDate },
}

pub struct Meals {
	calendar: Calendar,
	calendar_state: CalendarState,
	meals_chooser: MealsChooser,
	meals_database: Arc<RestDatabase<MealPlan>>,
	meals_list: MealsList,
	meals_list_menu: ScrollableMenu,
	random_meal_chooser: RandomMealChooser,
	shopping_list: ShoppingList,
}

#[derive(Clone, Debug)]
pub enum MealsMessage {
	AddMonth(isize),
	CloseOpenMeal {
		date: NaiveDate,
		time: Time,
	},
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
	ResetChooser,
	Scrollable(ScrollableMenuMessage),
	SetCalendarState(CalendarState),
	SelectMealForDate {
		date: NaiveDate,
		id: Uuid,
	},
	SearchMeal(String),
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
	Updated,
}

impl From<ScrollableMenuMessage> for MealsMessage {
	fn from(message: ScrollableMenuMessage) -> Self {
		MealsMessage::Scrollable(message)
	}
}

impl Meals {
	pub fn new() -> (Self, Task<Message>) {
		let (meals_database, mut meals_receiver) = block_on(RestDatabase::new(
			&std::env::var("BANSHEELONG2_GET_ALL_MEALS_URL").unwrap(),
			&std::env::var("BANSHEELONG2_REPLACE_URL").unwrap(),
			&std::env::var("BANSHEELONG2_WS_URL").unwrap(),
		));

		let meals_database = Arc::new(meals_database);

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
				meals_database: meals_database.clone(),
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
				Task::stream(stream::channel(100, |mut output| async move {
					loop {
						select! {
							_ = meals_database.recv_loop().fuse() => {}
							message = meals_receiver.recv().fuse() => {
								let Some(message) = message else {
									unreachable!();
								};

								match message {
									MealPlanMessage::Update => output.send(Message::Meals(MealsMessage::Updated)).await.unwrap()
								}
							}
						}
					}
				})),
				Task::done(Message::Meals(MealsMessage::Updated)),
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

				let meals_database = self.meals_database.clone();
				Task::batch([
					Task::future(async move {
						meals_database.save().await;
						Message::Noop
					}),
					Task::done(Message::Meals(MealsMessage::CloseOpenMeal { date, time })),
				])
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

				let meals_database = self.meals_database.clone();
				Task::batch([
					Task::future(async move {
						meals_database.save().await;
						Message::Noop
					}),
					Task::done(Message::Meals(MealsMessage::SetCalendarState(
						CalendarState::Calendar,
					))),
				])
			}
			MealsMessage::SetCalendarState(state) => {
				let old_state = self.calendar_state.clone();
				self.calendar_state = state;

				match &self.calendar_state {
					CalendarState::Chooser { date } => {
						if let CalendarState::ChooserSearch { .. } = old_state {
							return Task::none();
						};

						self.meals_chooser.set_current_date(date.clone());
						Task::batch([
							Task::done(Message::Meals(MealsMessage::Scrollable(
								ScrollableMenuMessage::Reset {
									id: self.meals_chooser.menu.id.clone(),
								},
							))),
							Task::done(Message::Meals(MealsMessage::ResetChooser)),
						])
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

				let meals_database = self.meals_database.clone();
				Task::future(async move {
					meals_database.save().await;
					Message::Noop
				})
			}
			MealsMessage::ToggleOpenMeal { .. } | MealsMessage::CloseOpenMeal { .. } => {
				self.meals_list.update(event)
			}
			MealsMessage::ToggleOpenMealInChooser { .. }
			| MealsMessage::ResetChooser
			| MealsMessage::SearchMeal(..) => self.meals_chooser.update(event),
			MealsMessage::ToggleShoppingListItem { .. }
			| MealsMessage::GenerateShoppingList
			| MealsMessage::PruneShoppingList { .. } => self.shopping_list.update(event),
			MealsMessage::Updated => {
				let meals_database = self.meals_database.clone();
				/*Task::future(async move {
					meals_database.load().await;
					log::info!("Updated meals database");
					Message::Noop
				})*/

				Task::stream(stream::channel(100, |mut output| async move {
					let meal_plan = meals_database.get();
					let old_meals = meal_plan.planned_meals.clone();
					drop(meal_plan);

					meals_database.load().await;
					log::info!("Updated meals database");

					let meal_plan = meals_database.get();
					for (date, planned_meals1) in old_meals.iter() {
						let Some(planned_meals2) = meal_plan.planned_meals.get(&date) else {
							for planned_meal in planned_meals1 {
								output
									.send(Message::Meals(MealsMessage::CloseOpenMeal {
										date: planned_meal.date,
										time: planned_meal.time,
									}))
									.await
									.unwrap();
							}

							continue;
						};

						for planned_meal1 in planned_meals1.iter() {
							let mut found = false;
							for planned_meal2 in planned_meals2.iter() {
								if planned_meal1 == planned_meal2 {
									found = true;
								}
							}

							if !found {
								output
									.send(Message::Meals(MealsMessage::CloseOpenMeal {
										date: *date,
										time: planned_meal1.time,
									}))
									.await
									.unwrap();
							}
						}
					}
				}))
			}
		}
	}

	pub fn view(&self) -> Element<MealsMessage> {
		let mut column = column(vec![]).spacing(10);
		if let Some(shopping_list) = self.shopping_list.view() {
			column = column.push(shopping_list);
		}

		let meal_plan = self.meals_database.get();
		let size = meal_plan
			.planned_meals
			.values()
			.fold(0, |prev, meals| prev + meals.len());

		let min_height = (320u16).saturating_sub((40 * size + 10 * (size - 1)) as u16);
		drop(meal_plan);

		column = column.push(self.meals_list.view());

		let width = match self.calendar_state {
			CalendarState::ChooserSearch { .. } => 344,
			_ => 400,
		};

		row!(
			container(self.meals_list_menu.view(
				column.into(),
				vec![button(
						container(text!("Generate shopping list"))
							.align_x(Alignment::Center)
							.width(Length::Fill),
					)
					.on_press(MealsMessage::GenerateShoppingList)
					.width(Length::Fill)
					.style(|theme, _status| primary_button(theme))
					.into()],
				min_height.saturating_sub(10),
			))
			.width(width)
			.height(Length::Fill),
			match self.calendar_state {
				CalendarState::Calendar => self.calendar.view(),
				CalendarState::Chooser { .. } => self.meals_chooser.view(false),
				CalendarState::ChooserSearch { .. } => self.meals_chooser.view(true),
				CalendarState::RandomChooser { .. } => self.random_meal_chooser.view(),
			}
		)
		.spacing(16)
		.align_y(Alignment::Center)
		.width(720)
		.into()
	}
}
