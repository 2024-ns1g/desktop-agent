use connection::{establish_ws_connection, verify_otp, Event};
use eframe::egui;
use once_cell::sync::Lazy;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;

mod connection;

#[derive(Default)]
struct AppState {
    // ユーザー入力パラメータ
    primary_server_address: String,
    session_server_address: String,

    otp: String,
    agent_name: String,

    // 取得した値
    session_id: String,
    token: String,

    // 接続状態フラグ
    connected: bool,

    // UIに表示するステータスメッセージ
    status_message: String,

    // スライド情報
    current_slide_index: usize,
    total_slide_count: usize,

    // イベント伝達用のチャンネル
    event_sender: Option<Sender<Event>>,
    event_receiver: Option<Receiver<Event>>,
}

impl AppState {
    pub fn connect_to_session(&mut self) {
        let client = reqwest::Client::new();
        let base_url = self.primary_server_address.clone();
        let otp = self.otp.clone();
        let sender = self
            .event_sender
            .take()
            .expect("Failed to get event sender");

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            let result = rt.block_on(verify_otp(&client, &base_url, &otp));

            match result {
                Ok(response) => {
                    {
                        let mut state = APP_STATE.lock().unwrap();
                        state.session_id = response.session_id;
                        state.token = response.token;
                        state.status_message = "OTP verified successfully.".to_owned();
                    }
                    // 成功後にWebSocket接続を確立
                    APP_STATE.lock().unwrap().establish_ws_connection(sender);
                }
                Err(e) => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.status_message = format!("OTP Verification Failed: {}", e);
                }
            }
        });
    }

    pub fn establish_ws_connection(&mut self, sender: Sender<Event>) {
        let session_id = self.session_id.clone();
        let token = self.token.clone();
        let agent_name = self.agent_name.clone();
        let session_server_address = self.session_server_address.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(establish_ws_connection(
                &session_server_address,
                &session_id,
                &token,
                &agent_name,
                sender,
            ));
        });
    }
}

static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| {
    let (sender, receiver) = mpsc::channel();
    Mutex::new(AppState {
        event_sender: Some(sender),
        event_receiver: Some(receiver),
        ..Default::default()
    })
});

fn main() -> eframe::Result {
    env_logger::init();

    let builder = egui::ViewportBuilder::default()
        .with_title("My egui App")
        .with_inner_size(egui::vec2(400.0, 300.0));

    let options = eframe::NativeOptions {
        viewport: builder,
        ..Default::default()
    };

    // ウィンドウを起動し、UIループを開始
    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        // 毎フレーム ui_main を呼び出す
        ui_main(ctx);
    })
}

fn ui_main(ctx: &egui::Context) {
    {
        let mut state = APP_STATE.lock().unwrap();

        // 受信チャンネルが存在する場合、イベントを処理
        if let Some(receiver) = &state.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    Event::KeyPress { key } => match key.as_str() {
                        "ArrowRight" => {
                            if state.current_slide_index < state.total_slide_count {
                                state.current_slide_index += 1;
                                state.status_message = "次のスライドに移動しました。".to_owned();
                            }
                        }
                        "ArrowLeft" => {
                            if state.current_slide_index > 0 {
                                state.current_slide_index -= 1;
                                state.status_message = "前のスライドに戻りました。".to_owned();
                            }
                        }
                        _ => {
                            state.status_message = format!("未対応のキー押下: {}", key);
                        }
                    },
                }
            }
        }
    }

    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        egui::Frame::default()
            .outer_margin(egui::vec2(0.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // 左側
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.heading("My egui App");
                    });

                    // 右側
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Disconnect").clicked() {
                            // 切断ロジック（例）
                            state.connected = false;
                            state.status_message = "切断されました".to_owned();
                        }
                    });
                });
            });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add_space(8.0);

        // 接続状態に基づいてUIを分岐
        if state.connected {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.heading("Connected");
                    ui.label(format!(
                        "Slide: {}/{}",
                        state.current_slide_index, state.total_slide_count
                    ));
                },
            );
        } else {
            ui.with_layout(
                egui::Layout {
                    main_dir: egui::Direction::TopDown,
                    main_align: egui::Align::Center,
                    cross_align: egui::Align::Center,
                    ..Default::default()
                },
                |ui| {
                    ui.heading("Connect");
                    ui.add_space(12.0);
                    egui::Grid::new("connect_grid")
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Address (Primary):");
                            ui.text_edit_singleline(&mut state.primary_server_address);
                            ui.end_row();

                            ui.label("Address (Session):");
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
                        // 既に接続は verify_otp 内で行われるため、ここでは不要
                    }
                },
            );
        }
    });

    egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // 左側に接続状態を表示
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.label(format!(
                    "Status: {}",
                    if state.connected {
                        "Connected"
                    } else {
                        "Not Connected"
                    }
                ));
            });

            // 右側にステータスメッセージを表示
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Message: {}", state.status_message));
            });
        });
    });
}
