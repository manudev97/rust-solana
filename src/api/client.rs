use reqwest::header;
use crate::error::SwapError;
use crate::models::coin_data::CoinData;

/// Creates an HTTP client with Pump.fun specific headers
pub async fn create_http_client() -> Result<reqwest::Client, SwapError> {
    let client = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:125.0) Gecko/20100101 Firefox/125.0",
        )
        .default_headers({
            let mut headers = header::HeaderMap::new();
            headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));
            headers.insert(
                header::ACCEPT_LANGUAGE,
                header::HeaderValue::from_static("en-US,en;q=0.5"),
            );
            headers.insert(
                header::REFERER,
                header::HeaderValue::from_static("https://www.pump.fun/"),
            );
            headers.insert(
                header::ORIGIN,
                header::HeaderValue::from_static("https://www.pump.fun"),
            );
            headers
        })
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    Ok(client)
}

/// Fetches token data from Pump.fun API
pub async fn get_coin_data(mint_str: &str) -> Result<CoinData, SwapError> {
    let client = create_http_client().await?;
    let url = format!("https://frontend-api-v3.pump.fun/coins/{}", mint_str);

    let response = client.get(&url).send().await?;

    if response.status().is_success() {
        let coin_data = response.json::<CoinData>().await?;
        Ok(coin_data)
    } else {
        let status = response.status();
        let error_text = response.text().await?;
        Err(SwapError::ApiError(format!(
            "API returned status {}: {}",
            status, error_text
        )))
    }
}