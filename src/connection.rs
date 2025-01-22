use enigo::{Direction::Click, Key, Keyboard, Settings};
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite;

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
    // dataオブジェクトを追加
    data: RegisterAgentMessageData<'a>,
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
                    "ArrowRight" => enigo.key(Key::RightArrow, Click).unwrap(),
                    "ArrowLeft" => enigo.key(Key::LeftArrow, Click).unwrap(),
                    _ => {}
                }
            });
        }
    }
}

pub async fn run_websocket(
    base_url: &str,
    session_id: &str,
    token: &str,
    agent_name: &str,
) -> Result<(), anyhow::Error> {
    // ws or wss
    let prefix = if base_url.starts_with("https") {
        "wss"
    } else {
        "ws"
    };
    let (mut ws_stream, _) =
        tokio_tungstenite::connect_async(format!("{}://{}/agent?sessionId={}", prefix, base_url, session_id))
            .await?;
    
    let register_message = serde_json::to_string(&RegisterAgentMessage {
        msg_type: "REGIST_AGENT",
        data: RegisterAgentMessageData {
            agent_name,
            agent_type: "SHOW_SLIDE_DESKTOP",
            token,
        },
    })?;

    ws_stream
        .send(tungstenite::Message::text(register_message))
        .await?;

    while let Some(msg) = ws_stream.next().await {
        match msg {
            Ok(msg) => {
                let event: Event = serde_json::from_str(&msg.to_string())?;
                handle_event(event).await;
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}


#[derive(Serialize)]
struct SessionInfoPageScript<'a> {
    content: &'a str,
}

#[derive(Serialize)]
struct SessionInfoPage<'a> {
    #[serde(rename = "pageId")]
    page_id: &'a str,
    title: &'a str,
    scripts: Vec<SessionInfoPageScript<'a>>,
}

#[derive(Serialize)]
struct SessionInfoAvailableVoteChoice<'a> {
    #[serde(rename = "choiceId")]
    choice_id: &'a str,
    title: &'a str,
    description: Option<&'a str>,
    color: Option<&'a str>,
}

#[derive(Serialize)]
struct SessionInfoAvailableVote<'a> {
    #[serde(rename = "voteId")]
    vote_id: &'a str,
    title: &'a str,
    description: Option<&'a str>,
    choices: Vec<SessionInfoAvailableVoteChoice<'a>>,
}

#[derive(Serialize)]
struct SessionInfoVote<'a> {
    #[serde(rename = "voteId")]
    vote_id: &'a str,
    #[serde(rename = "choiceId")]
    choice_id: &'a str,
    #[serde(rename = "voterId")]
    voter_id: &'a str,
}

#[derive(Serialize)]
struct SessionInfoState<'a> {
    #[serde(rename = "currentPage")]
    current_page: &'a i8,
    #[serde(rename = "currentVoteId")]
    available_vote_id: Option<&'a str>,
    votes: Vec<SessionInfoVote<'a>>,
}

#[derive(Serialize)]
struct SessionInfo<'a> {
    #[serde(rename = "sessionId")]
    session_id: &'a str,
    #[serde(rename = "slideId")]
    slide_id: &'a str,
    pages: Vec<SessionInfoPage<'a>>,
    #[serde(rename = "availableVotes")]
    available_votes: Vec<SessionInfoAvailableVote<'a>>,
    state: SessionInfoState<'a>,
}
