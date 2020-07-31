pub mod error;

use super::settings;
use error::FetchError;
use url::Url;

type FetchResult<T> = Result<T, FetchError>;

fn assert_within_size_limit(size: u64, limit: u64) -> FetchResult<()> {
    if size > limit {
        // The response is larger than the maximum allowed size. ERROR!!!
        Err(error::FetchError::MaxSizeExceeded(limit, size))
    } else {
        Ok(())
    }
}

pub async fn fetch_bytes(url: &Url, options: &settings::Fetch) -> FetchResult<Vec<u8>> {
    if let Some(host) = url.host_str() {
        if options.allowed_hosts.is_match(&host) {
            let res = reqwest::get(&url.to_string()).await?;

            let body_size = match res.content_length() {
                Some(x) => x,
                None => {
                    // If Reqwest can't determine the size of the input, nobody can! We must play it safe and ABORT!
                    return Err(FetchError::InvalidInput);
                }
            };

            assert_within_size_limit(body_size, options.max_size)?;

            let bytes = res.bytes().await?.to_vec();

            assert_within_size_limit(bytes.len() as u64, options.max_size)?;

            return Ok(bytes);
        }
    }

    Err(error::FetchError::IllegalHost(
        url.host_str().unwrap_or_else(|| "").to_string(),
    ))
}
