use bytes::Bytes;

pub async fn download_image(url: String) -> Option<Bytes> {
	let response = match reqwest::get(&url).await {
		Ok(response) => response,
		Err(error) => {
			log::error!("Could not fetch image ({}) {:?}", url, error);
			return None;
		}
	};

	match response.bytes().await {
		Ok(bytes) => Some(bytes),
		Err(error) => {
			log::error!("Could not encode image into bytes {:?}", error);
			None
		}
	}
}
