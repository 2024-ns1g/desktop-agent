use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio;

#[derive(Serialize)]
pub struct VerifyOtpRequest {
    pub otp: String,
}

#[derive(Deserialize)]
pub struct VerifyOtpResponse {
    pub session_id: String,
    pub token: String,
}

pub async fn verify_otp(
    client: &Client,
    base_url: &str,
    otp: &str,
) -> Result<VerifyOtpResponse, anyhow::Error> {
    let url = format!("{}/api/session/verify", base_url);
    let request = VerifyOtpRequest {
        otp: otp.to_string(),
    };

    let resp = client.post(&url).json(&request).send().await?;

    if resp.status().is_success() {
        let response = resp.json::<VerifyOtpResponse>().await?;
        Ok(response)
    } else {
        Err(anyhow::anyhow!("Failed to verify OTP"))
    }
}
