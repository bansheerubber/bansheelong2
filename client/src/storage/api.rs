use futures::{SinkExt, Stream};
use iced::stream;
use storage_server::JobStatusFlags;
use tokio::{
	net::TcpStream,
	time::{sleep, Duration},
};

use super::{component::StorageData, StorageMessage};

enum SocketState {
	Connected(TcpStream, String),
	Disconnected,
}

pub fn socket() -> impl Stream<Item = StorageMessage> {
	stream::channel(100, |mut output| async move {
		let mut state = SocketState::Disconnected;
		loop {
			match &mut state {
				SocketState::Connected(socket, buffer) => {
					if let Err(error) = socket.readable().await {
						log::error!("Storage TCP readable error {:?}", error);
						state = SocketState::Disconnected;
						sleep(Duration::from_secs(10)).await;
						continue;
					}

					let mut packet = Vec::new();
					match socket.try_read_buf(&mut packet) {
						Ok(0) => {
							log::error!("Storage TCP server is shut off");
							state = SocketState::Disconnected;
							sleep(Duration::from_secs(10)).await;
							continue;
						}
						Ok(_) => {
							let message = match String::from_utf8(packet) {
								Ok(string) => string,
								Err(error) => {
									log::error!("Storage TCP string conversion error {:?}", error);
									state = SocketState::Disconnected;
									sleep(Duration::from_secs(10)).await;
									continue;
								}
							};

							buffer.push_str(&message);
						}
						Err(error) if error.kind() == tokio::io::ErrorKind::WouldBlock => {
							sleep(Duration::from_millis(100)).await;
							continue;
						}
						Err(error) => {
							log::error!("Storage TCP read error {:?}", error);
							state = SocketState::Disconnected;
							sleep(Duration::from_secs(10)).await;
							continue;
						}
					}

					if !buffer.contains("\n") {
						continue;
					}

					let split = buffer.split("\n").collect::<Vec<_>>();

					let message = split[0];
					let parts = message
						.split(" ")
						.map(|x| match x.parse::<u64>() {
							Ok(value) => value,
							Err(_) => 0,
						})
						.collect::<Vec<_>>();

					*buffer = String::new();

					let result = output
						.send(StorageMessage::Update {
							data: StorageData {
								job_flags: if JobStatusFlags::from_bits(parts[0] as u64).is_none() {
									JobStatusFlags::GENERAL_ERROR
								} else {
									JobStatusFlags::from_bits(parts[0] as u64).unwrap()
								},

								used_size: parts[1],
								total_size: parts[2],

								btrfs_used_size: parts[3],
								btrfs_total_size: parts[4],
								btrfs_backup_count: parts[5],
							},
						})
						.await;

					if let Err(error) = result {
						log::error!("Storage TCP stream error {:?}", error);
					}
				}
				SocketState::Disconnected => {
					match TcpStream::connect("bansheestorage:3003").await {
						Ok(socket) => {
							state = SocketState::Connected(socket, String::new());
						}
						Err(error) => {
							log::error!("Storage TCP connection error {:?}", error);
							sleep(Duration::from_secs(10)).await;
						}
					}
				}
			}
		}
	})
}
