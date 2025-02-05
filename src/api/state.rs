use crate::models::state::SessionState;
use anyhow::Result;
use log::{error, info};
use reqwest::Client;

pub async fn get_session_state(
    client: &Client,
    base_url: &str,
    session_id: &str,
    token: &str,
) -> Result<SessionState, anyhow::Error> {
    let url = format!("{}/api/session/{}/agent/state", base_url, session_id);
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    if resp.status().is_success() {
        let response = resp.json::<SessionState>().await?;
        info!("Session state received successfully");
        Ok(response)
    } else {
        error!("Failed to get session state");
        Err(anyhow::anyhow!("Failed to get session state"))
    }
}
