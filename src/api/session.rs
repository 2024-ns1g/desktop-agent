use crate::models::session::SessionInfo;
use log::{error, info};
use reqwest::Client;
use anyhow::Result;

pub async fn get_session_info(
    client: &Client,
    base_url: &str,
    session_id: &str,
    token: &str,
) -> Result<SessionInfo, anyhow::Error> {
    let url = format!("{}/api/session/{}/agent/info", base_url, session_id);
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    if resp.status().is_success() {
        let response = resp.json::<SessionInfo>().await?;
        info!("Session info received successfully");
        Ok(response)
    } else {
        error!("Failed to get session info");
        Err(anyhow::anyhow!("Failed to get session info"))
    }
}
