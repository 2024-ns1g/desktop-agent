use enigo::{Key, Settings, Keyboard, Direction::Click};
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
    let url = format!("{}/session/agent/verify", base_url);
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

#[derive(Serialize)]
struct RegisterAgentMessage<'a> {
    #[serde(rename = "type")]
    msg_type: &'a str,
    #[serde(rename = "agentName")]
    agent_name: &'a str,
    #[serde(rename = "agentType")]
    agent_type: &'a str,
    token: &'a str,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Event {
    #[serde(rename = "KEY_PRESS")]
    KeyPress { key: String },
}

async fn handle_event(event: Event) {
    match event {
        Event::KeyPress { key } => {
            tokio::task::spawn_blocking(move || {
                let mut enigo = enigo::Enigo::new(&Settings::default()).unwrap();

                match key.as_str() {
                    "ArrowRight" => {
                        enigo.key(Key::RightArrow, Click).unwrap();
                    }
                    "ArrowLeft" => {
                        enigo.key(Key::LeftArrow, Click).unwrap();
                    }
                    _ => {}
                }
            });
        }
    }
}
