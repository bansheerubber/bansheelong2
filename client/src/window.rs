use iced::theme::palette::{Background, Danger, Extended, Pair, Primary, Secondary, Success};
use iced::theme::Palette;
use iced::widget::{container, row, text, Space};
use iced::{alignment, color, Element, Length, Subscription, Task, Theme};
use std::io::{Read, Write};
use std::path::Path;
use uuid::Uuid;

use crate::meals::{Meals, MealsMessage};
use crate::storage::{self, Storage, StorageMessage};
use crate::todos::{Todos, TodosMessage};
use crate::util::download_image;
use crate::weather::{Weather, WeatherMessage};
use crate::WINDOW_HEIGHT;

pub struct Window {
	meals: Meals,
	storage: Storage,
	todos: Todos,
	weather: Weather,
}

#[derive(Clone, Debug)]
pub enum Message {
	FetchImage { meal_id: Uuid, url: String },
	Meals(MealsMessage),
	Noop,
	RefetchWeather,
	Storage(StorageMessage),
	Todos(TodosMessage),
	Weather(WeatherMessage),
}

impl Window {
	pub fn new() -> (Self, Task<Message>) {
		let (meals, task) = Meals::new();
		(
			Self {
				meals,
				storage: Storage::new(),
				todos: Todos::new(),
				weather: Weather::new(),
			},
			Task::batch([Task::done(Message::RefetchWeather), task]),
		)
	}

	pub fn subscription(&self) -> Subscription<Message> {
		Subscription::batch([
			iced::time::every(std::time::Duration::from_secs(300)).map(|_| Message::RefetchWeather),
			Subscription::run(storage::socket).map(Message::Storage),
		])
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		let message = match message {
			Message::FetchImage { meal_id, url } => {
				let image_file_name = format!("./meals-images/{}", meal_id);
				let image_path = Path::new(&image_file_name);
				if image_path.exists() {
					let mut file = std::fs::File::open(image_path).unwrap();
					let mut buffer = vec![];
					file.read_to_end(&mut buffer).unwrap();

					return Task::done(Message::Meals(MealsMessage::Image {
						bytes: buffer.into(),
						url: url.clone(),
					}));
				}

				return Task::perform(download_image(url.clone()), move |bytes| match bytes {
					Some(bytes) => {
						let mut file = std::fs::File::create(&image_file_name).unwrap();
						file.write_all(&bytes).unwrap();

						Message::Meals(MealsMessage::Image {
							bytes,
							url: url.clone(),
						})
					}
					None => Message::Meals(MealsMessage::FailedImage { url: url.clone() }),
				});
			}
			Message::Meals(message) => return self.meals.update(message),
			Message::Noop => None,
			Message::RefetchWeather => {
				return Task::perform(crate::weather::dial(), move |result| {
					Message::Weather(WeatherMessage::ApiResult(result))
				})
			}
			Message::Storage(message) => return self.storage.update(message),
			Message::Todos(message) => self.todos.update(message),
			Message::Weather(message) => self.weather.update(message),
		};

		if let Some(message) = message {
			Task::done(message)
		} else {
			Task::none()
		}
	}

	pub fn view(&self) -> Element<Message> {
		container(row![
			self.weather.view().map(Message::Weather),
			container(
				container(text!(""))
					.style(|theme: &Theme| theme.extended_palette().background.weak.color.into())
					.width(2)
					.height(WINDOW_HEIGHT - 50.0)
			)
			.height(Length::Fill)
			.padding([0, 25])
			.align_y(alignment::Vertical::Center),
			// self.todos.view().map(Message::Todos),
			self.meals.view().map(Message::Meals),
			Space::with_width(20),
			self.storage.view().map(Message::Storage),
		])
		.width(Length::Fill)
		.height(WINDOW_HEIGHT)
		.into()
	}

	pub fn theme(&self) -> Theme {
		Theme::custom_with_fn(
			"bansheetheme".into(),
			Palette {
				background: color!(0x38263F), // #38263F
				text: color!(0xFFDDF3),       // #FFDDF3
				primary: color!(0xE059E0),    // #E059E0
				success: color!(0x2CBA60),    // #2CBA60
				danger: color!(0xDD4460),     // #DD4460
			},
			|palette| Extended {
				background: Background {
					strong: Pair::new(color!(0x322238), palette.text), // #322238
					base: Pair::new(palette.background, palette.text),
					weak: Pair {
						color: color!(0x583C63), // #583C63
						text: color!(0xFFB3FF),  // #FFB3FF
					},
				},
				primary: Primary {
					strong: Pair::new(color!(0xFF66FF), palette.text), // #FF66FF
					base: Pair::new(palette.primary, palette.text),
					weak: Pair::new(color!(0x8A76E0), palette.text), // #8A76E0
				},
				secondary: Secondary {
					strong: Pair::new(color!(0x58B7CE), palette.text), // #58B7CE
					base: Pair::new(color!(0xDBCD51), palette.text),   // #DBCD51
					weak: Pair::new(color!(0xD796F2), palette.text),   // #D796F2
				},
				success: Success {
					strong: Pair::new(palette.success, palette.text),
					base: Pair::new(palette.success, palette.text),
					weak: Pair::new(palette.success, palette.text),
				},
				danger: Danger {
					strong: Pair::new(palette.danger, palette.text),
					base: Pair::new(palette.danger, palette.text),
					weak: Pair::new(palette.danger, palette.text),
				},
				is_dark: true,
			},
		)
	}
}
