use enigo::{Direction::Click, Key, Keyboard, Settings};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::WebSocketStream;

#[derive(Debug)]
pub struct WsHandle {
    shutdown_tx: oneshot::Sender<()>,
}

impl WsHandle {
    pub fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }
}

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

#[derive(Serialize)]
struct RegisterAgentMessageData<'a> {
    #[serde(rename = "agentName")]
    agent_name: &'a str,
    #[serde(rename = "agentType")]
    agent_type: &'a str,
    token: &'a str,
}

#[derive(Serialize)]
struct RegisterAgentMessage<'a> {
    #[serde(rename = "requestType")]
    msg_type: &'a str,
    data: RegisterAgentMessageData<'a>,
}

// CHANGED: イベント型を拡張
#[derive(Deserialize)]
#[serde(tag = "type")]
enum Event {
    #[serde(rename = "KEY_PRESS")]
    KeyPress { key: String },
    #[serde(rename = "SLIDE_CHANGED")]
    SlideChanged {
        slide_index: usize,
        total_slides: usize,
    },
}

async fn handle_event(
    event: Event,
    sender: &std::sync::mpsc::Sender<crate::WsEvent>,
) -> Result<(), anyhow::Error> {
    match event {
        Event::KeyPress { key } => {
            tokio::task::spawn_blocking({
                let key = key.clone();
                move || {
                    let mut enigo = enigo::Enigo::new(&Settings::default()).unwrap();
                    match key.as_str() {
                        "ArrowRight" => enigo.key(Key::RightArrow, Click).unwrap(),
                        "ArrowLeft" => enigo.key(Key::LeftArrow, Click).unwrap(),
                        _ => {}
                    }
                }
            });
            sender.send(crate::WsEvent::KeyPressed(key))?;
        }
        Event::SlideChanged {
            slide_index,
            total_slides,
        } => {
            sender.send(crate::WsEvent::SlideChanged {
                index: slide_index,
                total: total_slides,
            })?;
        }
    }
    Ok(())
}

pub async fn run_websocket(
    base_url: &str,
    session_id: &str,
    token: &str,
    agent_name: &str,
    sender: std::sync::mpsc::Sender<crate::WsEvent>,
) -> Result<WsHandle, anyhow::Error> {
    let ws_base_url = base_url.replace("http", "ws");
    let (mut ws_stream, _) =
        tokio_tungstenite::connect_async(format!("{}/agent?sessionId={}", ws_base_url, session_id))
            .await?;

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // メインのWebSocket処理タスク
    let mut stream = ws_stream.split();
    let send_task = async {
        let register_message = serde_json::to_string(&RegisterAgentMessage {
            msg_type: "REGIST_AGENT",
            data: RegisterAgentMessageData {
                agent_name,
                agent_type: "SHOW_SLIDE_DESKTOP",
                token,
            },
        })?;
        stream
            .0
            .send(tungstenite::Message::text(register_message))
            .await?;
        Ok::<_, anyhow::Error>(stream.0)
    };

    let recv_task = async {
        while let Some(msg) = stream.1.next().await {
            let msg = msg?;
            let event: Event = serde_json::from_str(&msg.to_string())?;
            handle_event(event, &sender).await?;
        }
        Ok::<_, anyhow::Error>(())
    };

    tokio::spawn(async move {
        tokio::select! {
            result = send_task => {
                if let Err(e) = result {
                    error!("送信タスクエラー: {}", e);
                }
            }
            result = recv_task => {
                if let Err(e) = result {
                    error!("受信タスクエラー: {}", e);
                }
            }
            _ = shutdown_rx => {
                info!("切断要求を受信");
                ws_stream.close(None).await.ok();
                sender.send(crate::WsEvent::ConnectionClosed).ok();
            }
        }
    });

    Ok(WsHandle { shutdown_tx })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfoPageScript {
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfoPage {
    #[serde(rename = "pageId")]
    pub page_id: String,
    pub title: String,
    pub scripts: Vec<SessionInfoPageScript>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfoAvailableVoteChoice {
    #[serde(rename = "choiceId")]
    pub choice_id: String,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfoAvailableVote {
    #[serde(rename = "voteId")]
    pub vote_id: String,
    pub title: String,
    pub description: Option<String>,
    pub choices: Vec<SessionInfoAvailableVoteChoice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfoVote {
    #[serde(rename = "voteId")]
    pub vote_id: String,
    #[serde(rename = "choiceId")]
    pub choice_id: String,
    #[serde(rename = "voterId")]
    pub voter_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfoState {
    #[serde(rename = "currentPage")]
    pub current_page: i8,
    #[serde(rename = "currentVoteId")]
    pub available_vote_id: Option<String>,
    pub votes: Vec<SessionInfoVote>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfo {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "slideId")]
    pub slide_id: String,
    pub title: String,
    pub pages: Vec<SessionInfoPage>,
    #[serde(rename = "availableVotes")]
    pub available_votes: Vec<SessionInfoAvailableVote>,
    pub state: SessionInfoState,
}

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
