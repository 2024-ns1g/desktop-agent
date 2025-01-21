use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use tokio;

#[derive(Serialize)]
struct VerifyOtpRequest {
    otp: String,
}

#[derive(Deserialize)]
struct VerifyOtpResponse {
    session_id: String,
    token: String,
}

async fn verify_otp(
    client: &Client,
    base_url: &str,
    otp: &str,
) -> Result<VerifyOtpResponse, reqwest::Error> {
    let url = format!("{}/api/session/verify", base_url);
    let request = VerifyOtpRequest {
        otp: otp.to_string(),
    };

    let resp = client.post(&url).json(&request).send().await?;

    if resp.status().is_success() {
        let resp = resp.json::<VerifyOtpResponse>().await?;
        Ok(resp)
    } else {
        return Err(std::error
    }
}
