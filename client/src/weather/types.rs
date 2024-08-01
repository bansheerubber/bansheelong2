use std::cmp::{max, min};

use chrono::{DateTime, Local, TimeZone, Timelike};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct OneAPIError {
	pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OneAPIResponse {
	pub current: CurrentDatum,
	pub hourly: Vec<HourlyDatum>,
	pub daily: Vec<DailyDatum>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SkyDescription {
	pub id: u16,
	pub main: String,
	pub description: String,
	pub icon: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DailyTemperature {
	pub morn: f32,
	pub day: f32,
	pub eve: f32,
	pub night: f32,
	pub min: f32,
	pub max: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DailyFeelsLike {
	pub morn: f32,
	pub day: f32,
	pub eve: f32,
	pub night: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DailyDatum {
	pub feels_like: DailyFeelsLike,
	pub sunrise: u64,
	pub sunset: u64,
	pub temp: DailyTemperature,

	pub dt: i64,
	pub pressure: u16,
	pub humidity: u16,
	pub dew_point: f32,
	pub uvi: f32,
	pub clouds: u16,
	pub wind_speed: f32,
	pub wind_deg: u16,
	pub weather: Vec<SkyDescription>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HourlyDatum {
	pub pop: f32,
	pub visibility: u16,
	pub temp: f32,
	pub feels_like: f32,

	pub dt: i64,
	pub pressure: u16,
	pub humidity: u16,
	pub dew_point: f32,
	pub uvi: f32,
	pub clouds: u16,
	pub wind_speed: f32,
	pub wind_deg: u16,
	pub weather: Vec<SkyDescription>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentDatum {
	pub sunrise: u64,
	pub sunset: u64,
	pub visibility: u16,
	pub temp: f32,
	pub feels_like: f32,

	pub dt: i64,
	pub pressure: u16,
	pub humidity: u16,
	pub dew_point: f32,
	pub uvi: f32,
	pub clouds: u16,
	pub wind_speed: f32,
	pub wind_deg: u16,
	pub weather: Vec<SkyDescription>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct TemperatureDatum {
	pub time: u16,
	pub temperature: u16,
}

impl TemperatureDatum {
	pub fn get_temperature(&self) -> String {
		format!("{}°", self.temperature)
	}

	pub fn get_time(&self) -> String {
		if self.time == 12 {
			format!("{} PM", self.time)
		} else if self.time == 0 {
			String::from("12 AM")
		} else if self.time > 12 {
			format!("{} PM", self.time - 12)
		} else {
			format!("{} AM", self.time)
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct DailyStatus {
	pub current: TemperatureDatum,
	pub day: String,
	pub humidity: u16,
	pub icon: String,
	pub times: [TemperatureDatum; 3],
	pub uv: u16,
	pub wind: u16,
}

impl Into<DailyStatus> for &OneAPIResponse {
	fn into(self) -> DailyStatus {
		let mut next_time: u16 = (convert_to_time(self.current.dt)
			.format("%H")
			.to_string()
			.parse::<u16>()
			.unwrap() + 2)
			% 24;

		let mut times = [TemperatureDatum::default(); 3];
		let mut index = 0;
		let mut status_index = 0;
		while status_index < 3 {
			let time: u16 = convert_to_time(self.hourly[index].dt)
				.format("%H")
				.to_string()
				.parse()
				.unwrap();

			if time == next_time {
				times[status_index] = TemperatureDatum {
					time,
					temperature: self.hourly[index].temp as u16,
				};

				next_time = (time + 3) % 24;
				status_index += 1;
			}
			index += 1;
		}

		DailyStatus {
			current: TemperatureDatum {
				time: 0,
				temperature: self.current.temp as u16,
			},
			day: convert_to_time(self.current.dt).format("%A").to_string(),
			humidity: self.current.humidity,
			icon: decode_icon(
				self.current.weather[0].id,
				self.current.clouds > 50,
				is_day(),
			),
			times,
			uv: max(min(self.current.uvi as u16, 11), 1),
			wind: self.current.wind_speed as u16,
		}
	}
}

impl Into<DailyStatus> for &DailyDatum {
	fn into(self) -> DailyStatus {
		DailyStatus {
			current: TemperatureDatum {
				time: 0,
				temperature: self.temp.max as u16,
			},
			day: convert_to_time(self.dt).format("%A").to_string(),
			humidity: self.humidity,
			icon: decode_icon(self.weather[0].id, self.clouds > 50, true),
			times: [
				TemperatureDatum {
					time: 8,
					temperature: self.temp.morn as u16,
				},
				TemperatureDatum {
					time: 12,
					temperature: self.temp.day as u16,
				},
				TemperatureDatum {
					time: 12 + 8,
					temperature: self.temp.night as u16,
				},
			],
			uv: max(min(self.uvi as u16, 11), 1),
			wind: self.wind_speed as u16,
		}
	}
}

pub fn is_day() -> bool {
	let hour = Local::now().hour();
	return hour >= 4 && hour <= 19;
}

pub fn convert_to_time(time: i64) -> DateTime<Local> {
	Local.timestamp_opt(time, 0).unwrap()
}

pub fn decode_icon(id: u16, high_clouds: bool, day: bool) -> String {
	let status = match id {
		200..=202 => {
			if high_clouds {
				"thunderstorms-rain"
			} else if day {
				"thunderstorms-day-rain"
			} else {
				"thunderstorms-night-rain"
			}
		}
		210..=212 | 221 | 230..=232 => {
			if high_clouds {
				"thunderstorms"
			} else if day {
				"thunderstorms-day"
			} else {
				"thunderstorms-night"
			}
		}
		300..=302 | 313 | 314 | 321 => {
			if high_clouds {
				"drizzle"
			} else if day {
				"partly-cloudy-day-drizzle"
			} else {
				"partly-cloudy-night-drizzle"
			}
		}
		310..=312 | 500..=504 | 511 | 520..=522 | 530 => {
			if high_clouds {
				"rain"
			} else if day {
				"partly-cloudy-day-rain"
			} else {
				"partly-cloudy-night-rain"
			}
		}
		600..=602 | 615 | 616 | 620..=622 => {
			if high_clouds {
				"snow"
			} else if day {
				"partly-cloudy-day-snow"
			} else {
				"partly-cloudy-night-snow"
			}
		}
		611..=613 => {
			if high_clouds {
				"sleet"
			} else if day {
				"partly-cloudy-day-sleet"
			} else {
				"partly-cloudy-night-sleet"
			}
		}
		701 => "mist",
		711 => {
			if high_clouds {
				"smoke"
			} else if day {
				"partly-cloudy-day-smoke"
			} else {
				"partly-cloudy-night-smoke"
			}
		}
		721 => {
			if high_clouds {
				"haze"
			} else if day {
				"haze-day"
			} else {
				"haze-night"
			}
		}
		731 | 751 | 761 => {
			if high_clouds {
				"dust"
			} else if day {
				"dust-day"
			} else {
				"dust-night"
			}
		}
		741 => {
			if high_clouds {
				"fog"
			} else if day {
				"fog-day"
			} else {
				"fog-night"
			}
		}
		762 => "volcano",
		771 => "wind",
		781 => "tornado",
		800 => {
			if day {
				"clear-day"
			} else {
				"clear-night"
			}
		}
		801 | 802 => {
			if day {
				"partly-cloudy-day"
			} else {
				"partly-cloudy-night"
			}
		}
		803 => "cloudy",
		804 => "overcast",
		_other => "",
	}
	.to_string();

	return status;
}
