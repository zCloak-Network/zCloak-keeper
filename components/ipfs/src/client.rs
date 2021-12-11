use std::str;

pub struct IpfsClient {
	url_index: String,
}

impl IpfsClient {
	pub fn new(url: String) -> Self {
		Self { url_index: url }
	}

	pub async fn keep_fetch_proof(&self, proofid: &[u8]) -> Result<Vec<u8>, reqwest::Error> {
		let url_index = &self.url_index;
		let url = url_index.to_owned() + str::from_utf8(proofid).unwrap();

		log::debug!("file which on ipfs, url is {:?}", url);

		let mut body = String::new();

		loop {
			let maybe_response = reqwest::get(&url).await;
			if let Ok(response) = maybe_response {
				body = response.text().await?;
				break
			}
		}

		let body = body.as_bytes().to_owned();
		Ok(body)
	}
}
