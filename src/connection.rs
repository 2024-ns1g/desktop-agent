use enigo::{Direction::Click, Key, Keyboard, Settings};
use futures_util::{SinkExt, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite;
use log::{debug, info, error};
use std::{sync::mpsc::Sender, thread::sleep, time::Duration};

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

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Event {
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
                        enigo.key(Key::RightArrow, Click);
                        info!("Right arrow key pressed");
                    }
                    "ArrowLeft" => {
                        enigo.key(Key::LeftArrow, Click);
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
    event_sender: Sender<Event>,
) -> Result<(), anyhow::Error> {
    loop {
        debug!("WebSocketに接続を試みます: {}", base_url);
        match tokio_tungstenite::connect_async(format!(
            "{}/agent?sessionId={}",
            base_url, session_id
        ))
        .await
        {
            Ok((ws_stream, _)) => {
                info!("WebSocket接続が確立されました");
                let (mut write, read) = ws_stream.split();

                // 登録メッセージを送信
                let register_message = RegisterAgentMessage {
                    msg_type: "REGISTER_AGENT",
                    agent_name,
                    agent_type: "SHOW_SLIDE_DESKTOP",
                    token,
                };
                let register_message = serde_json::to_string(&register_message)?;
                debug!("登録メッセージを送信します: {}", register_message);
                write
                    .send(tungstenite::Message::text(register_message))
                    .await?;

                // メッセージの読み取りと処理
                read.for_each(|msg| async {
                    match msg {
                        Ok(message) => {
                            if let Ok(text) = message.to_text() {
                                match serde_json::from_str::<Event>(text) {
                                    Ok(event) => {
                                        handle_event(event.clone()).await;
                                        if let Err(e) = event_sender.send(event) {
                                            error!("イベントの送信に失敗しました: {}", e);
                                        }
                                    }
                                    Err(e) => error!("イベントの解析に失敗しました: {}", e),
                                }
                            }
                        }
                        Err(e) => {
                            error!("WebSocketエラー: {}", e);
                        }
                    }
                })
                .await;
            }
            Err(e) => {
                error!("WebSocket接続に失敗しました: {}", e);
            }
        }

        info!("5秒後にWebSocketへの再接続を試みます...");
        sleep(Duration::from_secs(5));
    }
}
