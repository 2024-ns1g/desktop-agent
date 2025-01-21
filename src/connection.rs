use enigo::{Direction::Click, Key, Keyboard, Settings};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite;
use log::{debug, info, error};

#[derive(Serialize, Debug)]
pub struct VerifyOtpRequest {
    pub otp: String,
}

#[derive(Deserialize, Debug)]
pub struct VerifyOtpResponse {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub token: String,
}

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
    debug!("Request: {:?}", request);
    let resp = client.post(&url).json(&request).send().await?;

    if resp.status().is_success() {
        let response = resp.json::<VerifyOtpResponse>().await?;
        info!("OTP verified successfully: {:?}", response);
        Ok(response)
    } else {
        error!("Failed to verify OTP: {:?}", resp);
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

                debug!("Key event: {}", key);

                match key.as_str() {
                    "ArrowRight" => {
                        enigo.key(Key::RightArrow, Click).unwrap();
                        info!("Right arrow key pressed");
                    }
                    "ArrowLeft" => {
                        enigo.key(Key::LeftArrow, Click).unwrap();
                        info!("Left arrow key pressed");
                    }
                    _ => {}
                }
            });
        }
    }
}

pub async fn establish_ws_connection(
    base_url: &str,
    session_id: &str,
    token: &str,
    agent_name: &str,
) -> Result<(), anyhow::Error> {
    debug!("Establishing WebSocket connection to {}", base_url);
    let (ws_stream, _) = tokio_tungstenite::connect_async(format!(
        "{}/agent?sessionId={}",
        base_url, session_id
    ))
    .await?;
    info!("WebSocket connection established");
    let (mut write, read) = ws_stream.split();
    let register_message = RegisterAgentMessage {
        msg_type: "REGISTER_AGENT",
        agent_name,
        agent_type: "SHOW_SLIDE_DESKTOP",
        token,
    };
    let register_message = serde_json::to_string(&register_message).unwrap();
    debug!("Sending register message: {}", register_message);
    write
        .send(tungstenite::Message::text(register_message))
        .await?;
    tokio::task::spawn(read.for_each(|msg| async {
        let msg = msg.unwrap();
        let event: Event = serde_json::from_str(&msg.to_string()).unwrap();
        handle_event(event).await;
    }));
    Ok(())
}
