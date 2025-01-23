use connection::{get_session_info, run_websocket, verify_otp};
use eframe::egui;
use once_cell::sync::Lazy;
use std::sync::Mutex;

mod connection;

// CHANGED: WebSocketイベント用のenumを追加
#[derive(Debug)]
pub enum WsEvent {
    SlideChanged { index: usize, total: usize },
    KeyPressed(String),
    ConnectionEstablished,
}

#[derive(Default)]
struct AppState {
    primary_server_address: String,
    session_server_address: String,
    otp: String,
    agent_name: String,
    session_id: String,
    token: String,
    connected: bool,
    status_message: String,
    slide_name: String,
    current_slide_index: usize,
    total_slide_count: usize,
    // CHANGED: イベント受信用チャネルを追加
    ws_event_receiver: Option<std::sync::mpsc::Receiver<WsEvent>>,
    logs: Vec<String>,           // NEW: ログ用フィールド追加
    ws_handle: Option<WsHandle>, // NEW: WebSocketハンドル追加
}

impl AppState {
    pub fn connect_to_session(&mut self) {
        let client = reqwest::Client::new();
        let base_url = self.primary_server_address.clone();
        let otp = self.otp.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(verify_otp(&client, &base_url, &otp));

            match result {
                Ok(response) => {
                    {
                        let mut state = APP_STATE.lock().unwrap();
                        state.session_id = response.session_id;
                        state.token = response.token;
                        state.status_message = "OTP verified successfully.".to_owned();
                    }
                    APP_STATE.lock().unwrap().fetch_session_info();
                    APP_STATE.lock().unwrap().establish_ws_connection();
                }
                Err(e) => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.status_message = format!("OTP Verification Failed: {}", e);
                }
            }
        });
    }

    pub fn establish_ws_connection(&mut self) {
        let session_id = self.session_id.clone();
        let token = self.token.clone();
        let agent_name = self.agent_name.clone();
        let session_server_address = self.session_server_address.clone();

        // CHANGED: チャネルを作成
        let (sender, receiver) = std::sync::mpsc::channel();
        self.ws_event_receiver = Some(receiver);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let result = rt.block_on(run_websocket(
                &session_server_address,
                &session_id,
                &token,
                &agent_name,
                sender, // CHANGED: senderを渡す
            ));

            {
                let mut state = APP_STATE.lock().unwrap();
                state.connected = false;
                state.status_message = match result {
                    Ok(()) => "WebSocket connection closed.".to_owned(),
                    Err(e) => format!("WebSocket error: {}", e),
                };
            }
        });
    }

    pub fn fetch_session_info(&mut self) {
        let client = reqwest::Client::new();
        let base_url = self.session_server_address.clone();
        let session_id = self.session_id.clone();
        let token = self.token.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(get_session_info(&client, &base_url, &session_id, &token));
            match result {
                Ok(response) => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.slide_name = response.title;
                    state.current_slide_index = response.state.current_page as usize;
                    state.total_slide_count = response.pages.len();
                }
                Err(e) => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.status_message = format!("Failed to fetch session info: {}", e);
                }
            }
        });
    }

    pub fn disconnect(&mut self) {
        if let Some(handle) = self.ws_handle.take() {
            handle.shutdown(); // WebSocket切断実行
        }
        self.connected = false;
        self.status_message = "Disconnected".to_owned();
        self.logs.clear();
        self.ws_event_receiver = None;
    }
}

static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PresenStudio agent")
            .with_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_simple_native("PresenStudio agent", options, move |ctx, _frame| {
        ui_main(ctx);
    })
}

fn ui_main(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::light());

    // イベント処理用の一時リスト
    let mut pending_events = Vec::new();

    // イベント収集フェーズ（不変借用のみ）
    {
        let state = APP_STATE.lock().unwrap();
        if let Some(receiver) = &state.ws_event_receiver {
            while let Ok(event) = receiver.try_recv() {
                pending_events.push(event);
            }
        }
    }

    // 状態更新フェーズ（可変借用）
    {
        let mut state = APP_STATE.lock().unwrap();
        for event in pending_events {
            match event {
                WsEvent::SlideChanged { index, total } => {
                    state.current_slide_index = index;
                    state.total_slide_count = total;
                    state.slide_name = format!("Slide {}", index + 1);
                }
                WsEvent::KeyPressed(key) => {
                    state.status_message = format!("Key pressed: {}", key);
                }
                WsEvent::ConnectionEstablished => {
                    state.connected = true;
                    state.status_message = "WebSocket connected".to_owned();
                }
            }
        }
    }

    let mut state = APP_STATE.lock().unwrap();

    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        egui::Frame::default()
            .outer_margin(egui::vec2(0.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        if state.connected {
                            ui.label(&state.slide_name);
                        }
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Disconnect").clicked() {
                            state.disconnect(); // 切断処理をメソッド化
                        }
                    });
                });
            });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        if state.connected {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Session Info");
                    ui.label(format!(
                        "Slide: {}/{}",
                        state.current_slide_index, state.total_slide_count
                    ));

                    // NEW: ログ表示用スクロールエリア
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for log in &state.logs {
                                ui.label(log);
                            }
                        });
                });
            });
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.heading("Connect");
                ui.add_space(12.0);
                egui::Grid::new("connect_grid")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Primary Server:");
                        ui.text_edit_singleline(&mut state.primary_server_address);
                        ui.end_row();

                        ui.label("Session Server:");
                        ui.text_edit_singleline(&mut state.session_server_address);
                        ui.end_row();

                        ui.label("OTP:");
                        ui.text_edit_singleline(&mut state.otp);
                        ui.end_row();

                        ui.label("Agent Name:");
                        ui.text_edit_singleline(&mut state.agent_name);
                        ui.end_row();
                    });

                ui.add_space(12.0);
                if ui.button("Connect").clicked() {
                    state.connect_to_session();
                }
            });
        }
    });

    egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!(
                "Status: {}",
                if state.connected {
                    "Connected"
                } else {
                    "Not Connected"
                }
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(&state.status_message);
            });
        });
    });
}
