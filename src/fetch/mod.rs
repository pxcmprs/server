pub mod error;

use super::settings;
use error::FetchError;
use log::debug;
use url::Url;

type FetchResult<T> = Result<T, FetchError>;

/// Compares two numbers, returns `Err` if `size > limit`.
fn assert_within_size_limit(size: u64, limit: u64) -> FetchResult<()> {
    if size > limit {
        Err(error::FetchError::MaxSizeExceeded(limit, size))
    } else {
        Ok(())
    }
}

/// Download bytes from the specified URL. Will make sure that the size of the downloaded file doesn't exceed the specified limit.
pub async fn fetch_bytes(url: &Url, options: &settings::Fetch) -> FetchResult<Vec<u8>> {
    if let Some(host) = url.host_str() {
        if options.allowed_hosts.is_match(&host) {
            debug!("Fetching {}", url);

            let res = reqwest::get(&url.to_string()).await?;

            let body_size = match res.content_length() {
                Some(x) => x,
                None => {
                    // If Reqwest can't determine the size of the input, nobody can! We must play it safe and ABORT!
                    return Err(FetchError::InvalidInput);
                }
            };

            debug!("Asserting that the downloaded file isn't too large");

            assert_within_size_limit(body_size, options.max_size)?;

            let bytes = res.bytes().await?.to_vec();

            assert_within_size_limit(bytes.len() as u64, options.max_size)?;

            debug!("Downloaded {} bytes", body_size);

            return Ok(bytes);
        }
    }

    Err(error::FetchError::IllegalHost(
        url.host_str().unwrap_or_else(|| "").to_string(),
    ))
}
