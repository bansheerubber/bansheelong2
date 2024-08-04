use chrono::{Datelike, Days, NaiveDate, Weekday};
use iced::{
	color,
	widget::{button, column, container, row, text},
	Alignment, Border, Color, Element, Length, Shadow, Theme,
};
use meals_database::{Database, MealPlan};
use std::rc::Rc;

use crate::{
	meals::{CalendarState, MealsMessage},
	pt,
	widgets::circle,
	Message,
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
	meals_database: Rc<Database<MealPlan>>,
	month: u32,
	year: i32,
}

const MEAL_COLORS: [Color; 3] = [color!(0xDBCD51), color!(0x58B7CE), color!(0xE059E0)];

impl Calendar {
	pub fn new(meals_database: Rc<Database<MealPlan>>) -> Self {
		Self {
			meals_database,
			month: 8,
			year: 2024,
		}
	}

	pub fn width(&self) -> u16 {
		7 * DAY_SIZE + 6 * DAY_SPACING
	}

	pub fn update(&mut self, event: MealsMessage) -> Option<Message> {
		match event {
			_ => unreachable!(),
		}
	}

	pub fn view(&self) -> Element<MealsMessage> {
		let mut start = NaiveDate::from_ymd_opt(self.year, self.month, 1).unwrap();
		let month = start.month0();

		while start.weekday() != Weekday::Sun {
			start = start.checked_sub_days(Days::new(1)).unwrap();
		}

		let meal_plan = self.meals_database.get();
		let mut days = column(vec![]).spacing(DAY_SPACING);
		let mut iter = start.clone();
		while (iter.month() <= self.month && iter.year() == self.year) || iter.year() < self.year {
			let mut week = row(vec![]).spacing(DAY_SPACING);
			for _ in 0..7 {
				let meals = meal_plan.planned_meals.get(&iter);

				let mut bubbles = row![]
					.spacing(4)
					.align_y(Alignment::End)
					.height(Length::Fill);

				if let Some(meals) = meals {
					for meal_stub in meals.iter() {
						bubbles = bubbles.push(circle(MEAL_COLORS[meal_stub.time.as_usize()], 3.0));
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
				text!("{}", MONTH[month as usize])
					.size(pt(25))
					.width(Length::Fill)
					.center(),
				days
			]
			.align_x(Alignment::Center)
			.spacing(10),
		)
		.into()
	}
}
