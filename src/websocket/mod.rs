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
        log::info!("Shutting down WebSocket");
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
    // 1. 接続
    let ws_base_url = base_url.replace("http", "ws");
    let (ws_stream, _) =
        tokio_tungstenite::connect_async(format!("{}/agent?sessionId={}", ws_base_url, session_id))
            .await?;

    // 2. 接続成功イベントを送信
    sender
        .send(crate::models::events::Event::ConnectionEstablished)
        .unwrap();

    // 3. 送信 (Sink) と 受信 (Stream) を分離
    let (mut sink, mut stream) = ws_stream.split();

    // 4. AGENT登録メッセージを送信
    let register_message = serde_json::to_string(&RegisterAgentMessage {
        msg_type: "REGIST_AGENT",
        data: RegisterAgentMessageData {
            agent_name,
            agent_type: "SHOW_SLIDE_DESKTOP",
            token,
        },
    })?;
    sink.send(tungstenite::Message::text(register_message))
        .await?;

    // 5. shutdown 用チャネルを生成
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    // 6. 別タスクで読み取りループを回す
    tokio::spawn(async move {
        tokio::select! {
            // A: 受信ループ
            _ = async {
                while let Some(Ok(msg)) = stream.next().await {
                    // WebSocketメッセージを受け取ったらパースしてイベントを送信
                    if let Ok(event) = serde_json::from_str::<WsEvent>(&msg.to_string()) {
                        handle_event(event, &sender).await;
                    } else {
                        log::warn!("Received invalid message: {:?}", msg);
                    }
                }
            } => {},

            // B: shutdown リクエストを待つ
            _ = shutdown_rx => {
                log::info!("WebSocket shutdown requested");
            }
        }

        // ループを抜けたらコネクションを閉じる
        sink.close().await.ok();
    });

    // 7. 生成した shutdown_tx を格納したハンドルを返す
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

                    let new_page_chars: Vec<char> =
                        data.new_page_index.to_string().chars().collect();

                    for c in new_page_chars {
                        enigo.key(Key::Unicode(c), Click).unwrap();
                    }

                    enigo.key(Key::Return, Click).unwrap();
                }
            });

            sender
                .send(crate::models::events::Event::SlideChanged {
                    new_page_index: data.new_page_index,
                })
                .unwrap();
        }
    }
}
