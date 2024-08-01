use iced::{
	widget::{button, column, container, row, svg, text},
	Alignment, Border, Element, Length, Padding, Shadow, Theme,
};

use crate::{pt, Message, ICONS, NOTOSANS_BOLD, WINDOW_HEIGHT};

use super::types::{DailyStatus, OneAPIError, OneAPIResponse, TemperatureDatum};

pub struct Weather {
	button_style: Box<dyn Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style>,
	current_day: usize,
	data: Option<OneAPIResponse>,
	daily_statuses: [DailyStatus; 5],
	show_detailed: bool,
	width: u16,
}

#[derive(Clone, Debug)]
pub enum WeatherMessage {
	ApiResult(Result<OneAPIResponse, OneAPIError>),
	NextDay,
	PreviousDay,
	ToggleDetail,
}

impl Weather {
	pub fn new() -> Self {
		let button_style = Box::new(|theme: &Theme, _status: button::Status| button::Style {
			background: None,
			text_color: theme.palette().text,
			border: Border::default(),
			shadow: Shadow::default(),
		});

		Self {
			button_style,
			current_day: 0,
			data: None,
			daily_statuses: [
				DailyStatus::default(),
				DailyStatus::default(),
				DailyStatus::default(),
				DailyStatus::default(),
				DailyStatus::default(),
			],
			show_detailed: false,
			width: 402,
		}
	}

	fn view_temperature_and_time(&self, datum: &TemperatureDatum) -> Element<WeatherMessage> {
		column![
			text(datum.get_temperature()).size(pt(55)),
			text(datum.get_time()).size(pt(25))
		]
		.align_x(Alignment::Center)
		.width(self.width / 3)
		.into()
	}

	fn view_detailed(&self) -> Element<WeatherMessage> {
		let uv_index_handle = svg::Handle::from_path(format!(
			"weather-svgs/uv-index-{}.svg",
			self.daily_statuses[self.current_day].uv
		));

		let raindrop_handle = svg::Handle::from_path("weather-svgs/raindrop.svg");

		let wind_handle = svg::Handle::from_path("weather-svgs/wind.svg");

		button(
			row![
				container(svg(uv_index_handle).width(80))
					.width((self.width as f32 * 0.2) as u16)
					.align_x(Alignment::Center),
				container(
					row![
						svg(raindrop_handle).width(60).height(80),
						text(format!(
							"{}%",
							self.daily_statuses[self.current_day].humidity
						))
						.size(pt(55)),
					]
					.align_y(Alignment::Center)
				)
				.width((self.width as f32 * 0.4) as u16)
				.align_x(Alignment::Center),
				container(
					row![
						svg(wind_handle).width(80),
						text(format!("{}", self.daily_statuses[self.current_day].wind))
							.size(pt(55))
					]
					.align_y(Alignment::Center)
				)
				.width((self.width as f32 * 0.4) as u16)
				.align_x(Alignment::Center),
			]
			.width(Length::Fill)
			.align_y(Alignment::Center),
		)
		.on_press(WeatherMessage::ToggleDetail)
		.padding(Padding::default().top(6))
		.style(&self.button_style)
		.into()
	}

	fn weather_info(&self) -> Element<WeatherMessage> {
		let current_day = &self.daily_statuses[self.current_day];
		let handle = svg::Handle::from_path(format!("weather-svgs/{}.svg", current_day.icon));

		let previous_button = button(
			text(if self.current_day != 0 {
				"\u{e408}"
			} else {
				""
			})
			.width(40)
			.height(Length::Fill)
			.align_y(Alignment::Center)
			.size(60)
			.font(ICONS),
		)
		.padding(0)
		.width(40)
		.height(Length::Fill)
		.on_press(WeatherMessage::PreviousDay)
		.style(&self.button_style);

		let next_button = button(
			text(if self.current_day < 4 { "\u{e409}" } else { "" })
				.width(40)
				.height(Length::Fill)
				.align_y(Alignment::Center)
				.size(60)
				.font(ICONS),
		)
		.padding(0)
		.width(40)
		.height(Length::Fill)
		.on_press(WeatherMessage::NextDay)
		.style(&self.button_style);

		let day_info = row![
			svg(handle).width(180),
			column![
				text(current_day.current.get_temperature())
					.size(pt(70))
					.font(NOTOSANS_BOLD),
				container(text(current_day.day.clone()).size(pt(25))).padding([0, 7]),
			]
			.align_x(Alignment::Start),
		]
		.padding(0)
		.width(301)
		.align_y(Alignment::Center);

		row![previous_button, day_info, next_button,]
			.padding(0)
			.height(180)
			.align_y(Alignment::Center)
			.into()
	}

	pub fn update(&mut self, event: WeatherMessage) -> Option<Message> {
		match event {
			WeatherMessage::ApiResult(Ok(data)) => {
				log::info!("Got weather data {:#?}", data);
				self.data = Some(data);

				self.daily_statuses[0] = self.data.as_ref().unwrap().into();

				for i in 1..5 {
					self.daily_statuses[i] =
						self.data.as_ref().unwrap().daily.get(i).unwrap().into();
				}
			}
			WeatherMessage::ApiResult(Err(error)) => {
				log::error!("{:#?}", error);
				self.data = None;
			}
			WeatherMessage::NextDay => {
				if self.current_day < 4 {
					self.current_day += 1;
				} else {
					self.current_day = 4;
				}
			}
			WeatherMessage::PreviousDay => {
				if self.current_day > 1 {
					self.current_day -= 1;
				} else {
					self.current_day = 0;
				}
			}
			WeatherMessage::ToggleDetail => {
				self.show_detailed = !self.show_detailed;
			}
		};

		None
	}

	pub fn view(&self) -> Element<WeatherMessage> {
		if self.data.is_none() {
			return container(
				container(text!(""))
					.width(self.width - 40)
					.height(250)
					.style(|theme: &Theme| theme.extended_palette().background.strong.color.into()),
			)
			.width(self.width)
			.height(WINDOW_HEIGHT)
			.padding(Padding::default().left(40))
			.align_y(Alignment::Center)
			.into();
		}

		let current_day = &self.daily_statuses[self.current_day];

		container(column![
			self.weather_info(),
			if self.show_detailed {
				self.view_detailed()
			} else {
				button(
					row(current_day
						.times
						.iter()
						.map(|datum| self.view_temperature_and_time(datum)))
					.width(Length::Fill)
					.align_y(Alignment::Center),
				)
				.on_press(WeatherMessage::ToggleDetail)
				.style(&self.button_style)
				.into()
			},
		])
		.width(self.width)
		.height(WINDOW_HEIGHT)
		.padding(Padding::default().top(8).left(20))
		.into()
	}
}
