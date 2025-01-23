use connection::{get_session_info, run_websocket, verify_otp, WsHandle};
use eframe::egui;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

mod connection;

#[derive(Debug)]
pub enum WsEvent {
    SlideChanged { index: usize, total: usize },
    KeyPressed(String),
    ConnectionEstablished,
    ConnectionClosed,
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
    logs: Vec<String>,
    ws_handle: Option<WsHandle>,
    ws_event_receiver: Option<UnboundedReceiver<WsEvent>>,
}

impl AppState {
    pub fn connect_to_session(&mut self) {
        if self.connected {
            self.logs.push("既に接続済みです".to_string());
            return;
        }

        let client = reqwest::Client::new();
        let base_url = self.primary_server_address.clone();
        let otp = self.otp.clone();

        self.logs.push("OTP検証を開始...".to_string());
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(verify_otp(&client, &base_url, &otp));

            match result {
                Ok(response) => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.session_id = response.session_id;
                    state.token = response.token;
                    state.status_message = "OTP認証成功".to_owned();
                    state.logs.push("OTP認証に成功しました".to_string());
                    state.fetch_session_info();
                    state.establish_ws_connection();
                }
                Err(e) => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.status_message = format!("OTP認証失敗: {}", e);
                    state.logs.push(format!("OTP認証エラー: {}", e));
                }
            }
        });
    }

    pub fn establish_ws_connection(&mut self) {
        let session_id = self.session_id.clone();
        let token = self.token.clone();
        let agent_name = self.agent_name.clone();
        let session_server_address = self.session_server_address.clone();
        
        let (sender, receiver) = unbounded_channel();
        self.ws_event_receiver = Some(receiver);

        self.logs.push("WebSocket接続を開始...".to_string());
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            match rt.block_on(run_websocket(
                &session_server_address,
                &session_id,
                &token,
                &agent_name,
                sender,
            )) {
                Ok(handle) => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.ws_handle = Some(handle);
                    state.logs.push("WebSocket接続確立".to_string());
                }
                Err(e) => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.logs.push(format!("WebSocket接続エラー: {}", e));
                }
            }
        });
    }

    pub fn disconnect(&mut self) {
        if let Some(handle) = self.ws_handle.take() {
            handle.shutdown();
            self.logs.push("WebSocket切断要求を送信".to_string());
        }
        self.connected = false;
        self.status_message = "切断済み".to_owned();
        self.ws_event_receiver = None;
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
}

static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("PresenStudio agent")
            .with_inner_size([600.0, 400.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_simple_native("PresenStudio agent", options, move |ctx, _frame| {
        ui_main(ctx);
    })
}

fn ui_main(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::light());

    let mut pending_events = Vec::new();
    {
        let state = APP_STATE.lock().unwrap();
        if let Some(receiver) = &state.ws_event_receiver {
            while let Ok(event) = receiver.try_recv() {
                pending_events.push(event);
            }
        }
    }

    {
        let mut state = APP_STATE.lock().unwrap();
        for event in pending_events {
            match event {
                WsEvent::SlideChanged { index, total } => {
                    state.current_slide_index = index;
                    state.total_slide_count = total;
                    state.slide_name = format!("スライド {}", index + 1);
                    state.logs.push(format!("スライド変更: {}/{}", index, total));
                }
                WsEvent::KeyPressed(key) => {
                    state.status_message = format!("キー押下: {}", key);
                    state.logs.push(format!("キーイベント: {}", key));
                }
                WsEvent::ConnectionEstablished => {
                    state.connected = true;
                    state.status_message = "接続済み".to_owned();
                    state.logs.push("WebSocket接続確立".to_string());
                }
                WsEvent::ConnectionClosed => {
                    state.connected = false;
                    state.status_message = "接続終了".to_owned();
                    state.logs.push("WebSocket接続終了".to_string());
                }
            }
        }
    }

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
                            state.disconnect();
                        }
                    });
                });
            });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        let state = APP_STATE.lock().unwrap();
        if state.connected {
            ui.vertical(|ui| {
                ui.heading("セッション情報");
                ui.label(format!(
                    "現在のスライド: {}/{}",
                    state.current_slide_index + 1,
                    state.total_slide_count
                ));

                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        for log in &state.logs {
                            ui.label(log);
                        }
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
                if state.connected { "Connected" } else { "Not Connected" }
            ));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(&state.status_message);
            });
        });
    });
}
