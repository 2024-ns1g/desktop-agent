use crate::models::auth::{VerifyOtpRequest, VerifyOtpResponse};
use log::{debug, error, info};
use reqwest::Client;
use anyhow::Result;

pub async fn verify_otp(
    client: &Client,
    base_url: &str,
    otp: &str,
) -> Result<VerifyOtpResponse, anyhow::Error> {
    debug!("Verifying OTP: {}", otp);
    let url = format!("{}/session/agent/verify", base_url);
    let request = VerifyOtpRequest {
        otp: otp.to_string(),
    };
    let resp = client.post(&url).json(&request).send().await?;

    if resp.status().is_success() {
        let response = resp.json::<VerifyOtpResponse>().await?;
        info!("OTP verified successfully");
        Ok(response)
    } else {
        error!("OTP verification failed");
        Err(anyhow::anyhow!("OTP verification failed"))
    }
}
