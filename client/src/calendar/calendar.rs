use chrono::{Datelike, Days, Local, Months, NaiveDate, Weekday};
use iced::{
	widget::{button, column, container, row, text, Space},
	Alignment, Border, Element, Length, Shadow, Task, Theme,
};
use meals_database::{MealPlan, RestDatabase};
use std::{collections::HashMap, sync::Arc};

use crate::{
	meals::{get_meal_color, CalendarState, MealsMessage},
	pt,
	styles::invisible_button,
	widgets::circle,
	Message, ICONS,
};

const MONTH: [&'static str; 12] = [
	"January",
	"February",
	"March",
	"April",
	"May",
	"June",
	"July",
	"August",
	"September",
	"October",
	"November",
	"December",
];

const DAY_SIZE: u16 = 40;
const DAY_FONT_SIZE: u32 = 18;
const DAY_SPACING: u16 = 4;

pub struct Calendar {
	meals_database: Arc<RestDatabase<MealPlan>>,
	start: NaiveDate,
}

impl Calendar {
	pub fn new(meals_database: Arc<RestDatabase<MealPlan>>) -> Self {
		let start = Local::now().date_naive().with_day(1).unwrap();

		Self {
			meals_database,
			start,
		}
	}

	pub fn width(&self) -> u16 {
		7 * DAY_SIZE + 6 * DAY_SPACING
	}

	pub fn update(&mut self, event: MealsMessage) -> Task<Message> {
		match event {
			MealsMessage::AddMonth(amount) => {
				if amount > 0 {
					self.start = self
						.start
						.checked_add_months(Months::new(amount as u32))
						.unwrap();
				} else {
					self.start = self
						.start
						.checked_sub_months(Months::new(amount.abs() as u32))
						.unwrap();
				}
			}
			_ => unreachable!(),
		}
		Task::none()
	}

	pub fn view(&self) -> Element<MealsMessage> {
		let current_year = Local::now().date_naive().year();
		let mut iter = self.start.clone();
		let month0 = iter.month0();

		while iter.weekday() != Weekday::Sun {
			iter = iter.checked_sub_days(Days::new(1)).unwrap();
		}

		let mut meal_id_to_color = HashMap::new();

		let meal_plan = self.meals_database.get();
		let mut days = column(vec![]).spacing(DAY_SPACING);
		while (iter.month() <= self.start.month() && iter.year() == self.start.year())
			|| iter.year() < self.start.year()
		{
			let mut week = row(vec![]).spacing(DAY_SPACING);
			for _ in 0..7 {
				let meals = meal_plan.planned_meals.get(&iter);

				let mut bubbles = row![]
					.spacing(4)
					.align_y(Alignment::End)
					.height(Length::Fill);

				if let Some(meals) = meals {
					for meal_stub in meals.iter() {
						bubbles = bubbles.push(if meal_stub.leftovers {
							let color =
								get_meal_color(&mut meal_id_to_color, &meal_stub.id).clone();

							container(
								container(Space::new(2.0, 6.0)).style(move |_theme| color.into()),
							)
							.width(6)
							.align_x(Alignment::Center)
						} else {
							container(circle(
								get_meal_color(&mut meal_id_to_color, &meal_stub.id),
								3.0,
							))
						});
					}
				}

				week = week.push(
					button(
						container(
							column![
								text!("{}", iter.day())
									.size(pt(DAY_FONT_SIZE))
									.width(Length::Fill),
								bubbles,
							]
							.align_x(Alignment::End)
							.height(Length::Fill),
						)
						.width(DAY_SIZE)
						.height(DAY_SIZE)
						.padding(5)
						.style(|theme: &Theme| {
							theme.extended_palette().background.strong.color.into()
						}),
					)
					.on_press(MealsMessage::SetCalendarState(
						CalendarState::RandomChooser { date: iter },
					))
					.padding(0)
					.style(|theme: &Theme, _status| button::Style {
						background: None,
						text_color: theme.palette().text,
						border: Border::default(),
						shadow: Shadow::default(),
					}),
				);

				iter = iter.checked_add_days(Days::new(1)).unwrap();
			}

			days = days.push(week);
		}

		container(
			column![
				row![
					button(text!("\u{e408}").size(pt(35)).font(ICONS))
						.on_press(MealsMessage::AddMonth(-1))
						.padding(0)
						.style(|theme, _status| invisible_button(theme)),
					text!(
						"{}{}",
						MONTH[month0 as usize],
						if current_year != self.start.year() {
							format!(" ({})", self.start.year())
						} else {
							"".into()
						}
					)
					.size(pt(25))
					.width(Length::Fill)
					.center(),
					button(text!("\u{e409}").size(pt(35)).font(ICONS))
						.on_press(MealsMessage::AddMonth(1))
						.padding(0)
						.style(|theme, _status| invisible_button(theme)),
				]
				.align_y(Alignment::Center),
				days
			]
			.align_x(Alignment::Center)
			.spacing(10),
		)
		.into()
	}
}
