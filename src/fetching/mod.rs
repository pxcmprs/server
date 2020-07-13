use crate::PxcmprsError;
use reqwest::{redirect, ClientBuilder};
use std::time::{Duration, Instant};

pub struct FetchResponse {
    pub bytes: Vec<u8>,
    pub duration: Duration,
}

pub async fn fetch_bytes(url: &String, max_size: u64) -> Result<FetchResponse, PxcmprsError> {
    let start = Instant::now();
    let client = ClientBuilder::new()
        .redirect(redirect::Policy::custom(|attempt| attempt.follow()))
        .build()?;
    let res = client.get(url).send().await?;

    let body_size = match res.content_length() {
        Some(x) => x,
        None => {
            // If Reqwest can't determine the size of the input, nobody can! We must play it safe and ABORT!
            return Err(PxcmprsError::InvalidInput);
        }
    };

    if body_size > max_size {
        return Err(PxcmprsError::MaxSizeExceeded(max_size, body_size));
    }

    let bytes = res.bytes().await?.to_vec();

    Ok(FetchResponse {
        bytes,
        duration: start.elapsed(),
    })
}
