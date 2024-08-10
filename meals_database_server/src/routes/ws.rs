use futures::{select, FutureExt, SinkExt, StreamExt};
use rocket::{get, State};
use rocket_ws::{
	frame::{CloseCode, CloseFrame},
	Channel, Message, WebSocket,
};
use serde_json::json;

use crate::Context;

#[get("/meals-events")]
pub async fn meals_events_stream(context: &State<Context>, ws: WebSocket) -> Channel<'static> {
	let mut receiver = context.meals_database.read().await.subscribe();
	ws.channel(move |mut stream| {
		Box::pin(async move {
			loop {
				select! {
					message = stream.next().fuse() => {
						log::info!("{:?}", message);

						match message {
							Some(_) => {},
							None => return Ok(()),
						}
					}
					message = receiver.recv().fuse() => {
						let Ok(message) = message else {
							break;
						};

						stream.send(Message::Text(json!(message).to_string())).await.unwrap();
					}
				}
			}

			stream
				.close(Some(CloseFrame {
					code: CloseCode::Error,
					reason: "got an error".into(),
				}))
				.await
				.unwrap();

			Ok(())
		})
	})
}
