use super::types::{OneAPIError, OneAPIResponse};

pub async fn dial() -> Result<OneAPIResponse, OneAPIError> {
	let client = reqwest::Client::new();
	let response_result = client
		.get("https://api.openweathermap.org/data/3.0/onecall")
		.query(&[
			("lat", "33.4484"),
			("lon", "-112.074"),
			("exclude", "minutely,alerts"),
			("appid", &std::env::var("BANSHEELONG2_WEATHER_APP_ID").unwrap()),
			("units", "imperial"),
		])
		.header(reqwest::header::CONTENT_TYPE, "application/json")
		.header(reqwest::header::ACCEPT, "application/json")
		.send()
		.await;

	if let Err(error) = response_result {
		return Err(OneAPIError {
			message: error.to_string(),
		});
	}

	let response = response_result.unwrap();
	match response.status() {
		reqwest::StatusCode::OK => {
			match response.json::<OneAPIResponse>().await {
				Ok(result) => return Ok(result),
				Err(error) => {
					return Err(OneAPIError {
						message: format!("Could not deserialize JSON: {:?}", error),
					})
				}
			};
		}
		other => {
			return Err(OneAPIError {
				message: format!("Error code {}", other),
			});
		}
	}
}
