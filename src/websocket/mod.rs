use crate::models::websocket::{RegisterAgentMessage, RegisterAgentMessageData, WsEvent};
use anyhow::Result;
use enigo::{Direction::Click, Key, Keyboard, Settings};
use futures_util::{SinkExt, StreamExt};
use log::{error, warn};
use tokio_tungstenite::tungstenite;

pub struct WsHandle {
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl WsHandle {
    pub fn shutdown(self) {
        let _ = self.shutdown_tx.send(());
    }
}

pub async fn run_websocket(
    base_url: &str,
    session_id: &str,
    token: &str,
    agent_name: &str,
    sender: std::sync::mpsc::Sender<crate::models::events::Event>,
) -> Result<WsHandle, anyhow::Error> {
    let ws_base_url = base_url.replace("http", "ws");
    let (mut ws_stream, _) =
        tokio_tungstenite::connect_async(format!("{}/agent?sessionId={}", ws_base_url, session_id))
            .await?;
    sender
        .send(crate::models::events::Event::ConnectionEstablished)
        .unwrap();

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
                let event: WsEvent = serde_json::from_str(&msg.to_string())?;
                handle_event(event, &sender).await;
            }
            Err(e) => return Err(e.into()),
        }
    }

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    // Task to handle incoming messages
    tokio::spawn(async move {
        tokio::select! {
            _ = async {
                while let Some(msg) = ws_stream.next().await {
                    if let Ok(msg) = msg {
                        if let Ok(event) = serde_json::from_str::<WsEvent>(&msg.to_string()) {
                            handle_event(event, &sender).await;
                        } else {
                            warn!("Received invalid message: {:?}", msg);
                        }
                    } else {
                        error!("Failed to receive message: {:?}", msg);
                        break;
                    }
                }
            } => {},
            _ = shutdown_rx => {
                log::info!("WebSocket shutdown requested");
            }
        }
        ws_stream.close(None).await.ok();
    });

    Ok(WsHandle { shutdown_tx })
}

async fn handle_event(
    event: WsEvent,
    sender: &std::sync::mpsc::Sender<crate::models::events::Event>,
) {
    match event {
        WsEvent::ChangeCurrentPage { data } => {
            tokio::task::spawn_blocking({
                move || {
                    let mut enigo = enigo::Enigo::new(&Settings::default()).unwrap();

                    let new_page_chars: Vec<char> = data.new_page_index.to_string().chars().collect();

                    for c in new_page_chars {
                        enigo.key(Key::Unicode(c), Click).unwrap();
                    }

                    enigo.key(Key::Return, Click).unwrap();
                }
            });

            sender.send(crate::models::events::Event::SlideChanged {
                new_page_index: data.new_page_index,
            }).unwrap();
        }
    }
}
