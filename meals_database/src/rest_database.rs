use async_tungstenite::{
	tokio::{connect_async, TokioAdapter},
	tungstenite::Message,
	WebSocketStream,
};
use futures::{executor::block_on, StreamExt};
use reqwest::header;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tokio::{
	net::TcpStream,
	sync::{
		mpsc::{self, Receiver, Sender},
		RwLock, RwLockReadGuard, RwLockWriteGuard,
	},
	time::sleep,
};

use crate::MealPlanMessage;

pub struct RestDatabase<T>
where
	T: Serialize + for<'a> Deserialize<'a> + Default,
{
	data: RwLock<T>,
	get_all_url: String,
	replace_url: String,
	sender: Sender<MealPlanMessage>,
	ws_socket: RwLock<WebSocketStream<TokioAdapter<TcpStream>>>,
	ws_url: String,
}

impl<T> RestDatabase<T>
where
	T: Serialize + for<'a> Deserialize<'a> + Default,
{
	pub async fn new(
		get_all_url: &str,
		replace_url: &str,
		ws_url: &str,
	) -> (Self, Receiver<MealPlanMessage>) {
		let ws_socket = Self::try_connect(ws_url).await;
		let (sender, receiver) = mpsc::channel(100);

		(
			Self {
				data: RwLock::new(T::default()),
				get_all_url: get_all_url.into(),
				replace_url: replace_url.into(),
				sender,
				ws_socket: RwLock::new(ws_socket),
				ws_url: ws_url.into(),
			},
			receiver,
		)
	}

	pub fn get(&self) -> RwLockReadGuard<T> {
		block_on(self.data.read())
	}

	pub fn get_mut(&self) -> RwLockWriteGuard<T> {
		block_on(self.data.write())
	}

	async fn try_connect(ws_url: &str) -> WebSocketStream<TokioAdapter<TcpStream>> {
		loop {
			match connect_async(ws_url).await {
				Ok((ws_socket, _)) => {
					log::info!("Connected to '{}'", ws_url);
					return ws_socket;
				}
				Err(error) => {
					log::error!("Failed to connect to '{}': {:?}", ws_url, error);
					sleep(Duration::from_secs(5)).await;
				}
			}
		}
	}

	pub async fn recv_loop(&self) {
		loop {
			let mut ws_socket = self.ws_socket.write().await;

			let result = match ws_socket.next().await {
				Some(message) => message,
				None => {
					log::error!("Stream became empty");

					sleep(Duration::from_secs(5)).await;
					*ws_socket = Self::try_connect(&self.ws_url).await;
					continue;
				}
			};

			let message = match result {
				Ok(message) => message,
				Err(error) => {
					log::error!("Encountered error during read: {:?}", error);

					if let Err(error) = ws_socket.close(None).await {
						log::error!("Failed to close socket: {:?}", error);
					}

					sleep(Duration::from_secs(5)).await;
					*ws_socket = Self::try_connect(&self.ws_url).await;
					continue;
				}
			};

			match message {
				Message::Text(text) => {
					let Ok(meal_plan_message) = serde_json::from_str(&text) else {
						log::error!("Could not decode '{}' into MealPlanMessage", text);
						continue;
					};

					if let Err(error) = self.sender.send(meal_plan_message).await {
						log::error!("Could not send message through channel: {:?}", error);
					}
				}
				Message::Binary(_) => log::error!("Unexpected binary message"),
				Message::Ping(_) => log::error!("Unexpected ping message"),
				Message::Pong(_) => log::error!("Unexpected pong message"),
				Message::Close(close_frame) => {
					log::error!("Unexpected close: {:?}", close_frame);

					sleep(Duration::from_secs(5)).await;
					*ws_socket = Self::try_connect(&self.ws_url).await;
					continue;
				}
				Message::Frame(_) => log::error!("Unexpected frame"),
			}
		}
	}

	pub async fn save(&self) {
		let data = self.get();
		let client = reqwest::Client::new();
		client
			.post(&self.replace_url)
			.body(json!(&*data).to_string())
			.header(header::CONTENT_TYPE, "application/json")
			.header(header::ACCEPT, "application/json")
			.header(
				header::AUTHORIZATION,
				format!(
					"Bearer {}",
					std::fs::read_to_string("./auth-token").unwrap().trim()
				),
			)
			.send()
			.await
			.unwrap();
	}

	pub async fn load(&self) {
		let client = reqwest::Client::new();
		let response = client
			.get(&self.get_all_url)
			.header(header::ACCEPT, "application/json")
			.header(
				header::AUTHORIZATION,
				format!(
					"Bearer {}",
					std::fs::read_to_string("./auth-token").unwrap().trim()
				),
			)
			.send()
			.await
			.unwrap()
			.json::<T>()
			.await
			.unwrap();

		let mut data = self.get_mut();
		*data = response;
	}
}
