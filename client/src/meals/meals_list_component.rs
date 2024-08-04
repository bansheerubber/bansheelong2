use chrono::{Datelike, NaiveDate};
use iced::{
	widget::{button, column, container, image, row, text, Space},
	Alignment, Border, Color, Element, Length, Padding, Shadow, Task, Theme,
};
use meals_database::{Database, MealInfo, MealPlan, MealStub, Time};
use std::{
	collections::{HashMap, HashSet},
	rc::Rc,
};
use uuid::Uuid;

use crate::{
	scrollable_menu::ScrollableMenu,
	styles::{danger_button, primary_button},
	widgets::circle,
	Message,
};

use super::{get_meal_color, meal_contents, MealsMessage};

pub struct MealsList {
	images: HashMap<String, image::Handle>,
	meals_database: Rc<Database<MealPlan>>,
	pub menu: ScrollableMenu,
	opened_meals: HashSet<(NaiveDate, Time)>,
	width: u16,
}

impl MealsList {
	pub fn new(meals_database: Rc<Database<MealPlan>>) -> (Self, Task<Message>) {
		let (menu, task) = ScrollableMenu::new();
		(
			Self {
				images: HashMap::new(),
				meals_database,
				opened_meals: HashSet::new(),
				width: 400,

				menu,
			},
			Task::batch([task]),
		)
	}

	pub fn width(&self) -> u16 {
		self.width
	}

	pub fn update(&mut self, event: MealsMessage) -> Task<Message> {
		match event {
			MealsMessage::FailedImage { .. } => Task::none(),
			MealsMessage::Image { bytes, url } => {
				self.images.insert(url, image::Handle::from_bytes(bytes));
				Task::none()
			}
			MealsMessage::ToggleOpenMeal { date, id, time } => {
				if self.opened_meals.contains(&(date, time)) {
					self.opened_meals.remove(&(date, time));
					Task::none()
				} else {
					let meal_plan = self.meals_database.get();
					self.opened_meals.insert((date, time));
					let url = meal_plan.all_meals.get(&id).unwrap().image.clone();

					Task::done(Message::FetchImage { meal_id: id, url })
				}
			}
			MealsMessage::Scrollable(message) => self.menu.update(message),
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
		let time = meal_stub.time;
		let color = get_meal_color(meal_id_to_color, &meal_info.id);
		if !self.opened_meals.contains(&(date, time)) {
			return button(
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
					text!("{}", meal_info.name)
				]
				.align_y(Alignment::Center)
				.spacing(10)
				.width(Length::Fill),
			)
			.on_press(MealsMessage::ToggleOpenMeal {
				date,
				id: meal_info.id,
				time,
			})
			.padding(10)
			.style(|theme: &Theme, _status| button::Style {
				background: Some(theme.extended_palette().background.strong.color.into()),
				text_color: theme.palette().text,
				border: Border::default(),
				shadow: Shadow::default(),
			})
			.into();
		}

		let mut meal_contents = meal_contents(
			meal_info,
			self.images.get(&meal_info.image),
			Some(meal_stub),
		);
		meal_contents = meal_contents.push(
			row![
				button(text!("Delete"))
					.on_press(MealsMessage::DeletePlannedMeal { date, time })
					.style(|theme, _status| danger_button(theme)),
				container(text!("")).width(Length::Fill),
				button(text!("Close"))
					.on_press(MealsMessage::ToggleOpenMeal {
						date,
						id: meal_info.id,
						time
					})
					.style(|theme, _status| primary_button(theme)),
			]
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

		container(self.menu.view(meals_list.into(), vec![]))
			.width(self.width)
			.height(Length::Fill)
			.into()
	}
}
