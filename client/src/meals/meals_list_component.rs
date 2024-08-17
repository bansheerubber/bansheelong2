use chrono::{Datelike, NaiveDate};
use iced::{
	widget::{button, column, container, image, row, text, Space},
	Alignment, Border, Color, Element, Length, Padding, Shadow, Task, Theme,
};
use meals_database::{MealInfo, MealPlan, MealStub, RestDatabase};
use std::{
	collections::{HashMap, HashSet},
	sync::Arc,
};
use uuid::Uuid;

use crate::{pt, styles::invisible_button, widgets::circle, Message, ICONS};

use super::{get_meal_color, meal_contents, MealsMessage};

pub struct MealsList {
	images: HashMap<String, image::Handle>,
	meals_database: Arc<RestDatabase<MealPlan>>,
	opened_meals: HashSet<(NaiveDate, Uuid)>,
	width: u16,
}

impl MealsList {
	pub fn new(meals_database: Arc<RestDatabase<MealPlan>>) -> (Self, Task<Message>) {
		(
			Self {
				images: HashMap::new(),
				meals_database,
				opened_meals: HashSet::new(),
				width: 400,
			},
			Task::none(),
		)
	}

	pub fn width(&self) -> u16 {
		self.width
	}

	pub fn update(&mut self, event: MealsMessage) -> Task<Message> {
		match event {
			MealsMessage::CloseOpenMeal { date, id } => {
				self.opened_meals.remove(&(date, id));
				Task::none()
			}
			MealsMessage::FailedImage { .. } => Task::none(),
			MealsMessage::Image { bytes, url } => {
				self.images.insert(url, image::Handle::from_bytes(bytes));
				Task::none()
			}
			MealsMessage::ToggleOpenMeal { date, id } => {
				if self.opened_meals.contains(&(date, id)) {
					self.opened_meals.remove(&(date, id));
					Task::none()
				} else {
					let meal_plan = self.meals_database.get();
					self.opened_meals.insert((date, id));
					let url = meal_plan.all_meals.get(&id).unwrap().image.clone();

					Task::done(Message::FetchImage { meal_id: id, url })
				}
			}
			_ => unreachable!(),
		}
	}

	fn view_meal(
		&self,
		meal_id_to_color: &mut HashMap<Uuid, Color>,
		meal_info: &MealInfo,
		meal_stub: &MealStub,
	) -> Element<MealsMessage> {
		let date = meal_stub.date;
		let id = meal_stub.id;
		let color = get_meal_color(meal_id_to_color, &meal_info.id);
		if !self.opened_meals.contains(&(date, id)) {
			return container(
				row![
					row![
						if meal_stub.leftovers {
							let color = get_meal_color(meal_id_to_color, &meal_stub.id).clone();
							container(
								container(Space::new(2.0, 6.0)).style(move |_theme| color.into()),
							)
							.width(6)
							.align_x(Alignment::Center)
							.align_y(Alignment::Center)
						} else {
							container(circle(get_meal_color(meal_id_to_color, &meal_stub.id), 3.0))
								.align_y(Alignment::Center)
						},
						text!("{}/{}/{}", date.month(), date.day(), date.year())
							.style(move |_theme| { text::Style { color: Some(color) } },),
					]
					.align_y(Alignment::Center)
					.spacing(5),
					button(text!("{}", meal_info.name))
						.on_press(MealsMessage::ToggleOpenMeal { date, id })
						.padding([10, 0])
						.style(|theme: &Theme, _status| button::Style {
							background: Some(
								theme.extended_palette().background.strong.color.into()
							),
							text_color: theme.palette().text,
							border: Border::default(),
							shadow: Shadow::default(),
						})
				]
				.align_y(Alignment::Center)
				.spacing(10)
				.width(Length::Fill),
			)
			.padding([0, 10])
			.style(|theme: &Theme| theme.extended_palette().background.strong.color.into())
			.into();
		}

		let meal_contents = meal_contents(
			meal_info,
			self.images.get(&meal_info.image),
			Some(meal_stub),
			Some(
				row![
					button(text("\u{e872}").font(ICONS).size(pt(30)))
						.on_press(MealsMessage::DeletePlannedMeal { date, id })
						.style(|theme, _status| invisible_button(theme))
						.padding(0),
					button(text("\u{ef6e}").font(ICONS).size(pt(30)))
						.on_press(MealsMessage::GenerateShoppingListForMeal {
							date,
							id: meal_info.id
						})
						.style(|theme, _status| invisible_button(theme))
						.padding(0),
					button(text("\u{e5ca}").font(ICONS).size(pt(30)))
						.on_press(MealsMessage::CompletePlannedMeal { date, id })
						.style(|theme, _status| invisible_button(theme))
						.padding(0),
					container(Space::new(0, 0)).width(Length::Fill),
					button(text("\u{e5cd}").font(ICONS).size(pt(30)))
						.on_press(MealsMessage::ToggleOpenMeal { date, id })
						.style(|theme, _status| invisible_button(theme))
						.padding(0),
				]
				.width(Length::Fill)
				.padding(Padding::default().bottom(5))
				.spacing(15)
				.into(),
			),
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

		let mut keys = meal_plan.planned_meals.keys().collect::<Vec<_>>();
		keys.sort();

		let mut meal_id_to_color = HashMap::new();
		for key in keys.iter() {
			let meals = meal_plan.planned_meals.get(key).unwrap();
			for meal_stub in meals.iter() {
				let meal_info = meal_plan.all_meals.get(&meal_stub.id).unwrap();
				meals_list =
					meals_list.push(self.view_meal(&mut meal_id_to_color, meal_info, meal_stub));
			}
		}

		container(meals_list).width(self.width).into()
	}
}
