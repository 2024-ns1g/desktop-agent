use connection::{establish_ws_connection, verify_otp};
use eframe::egui;
use once_cell::sync::Lazy;
use std::sync::Mutex;

mod connection;

#[derive(Default)]
struct AppState {
    // ユーザが入力するパラメータ
    primary_server_address: String,
    session_server_address: String,

    otp: String,
    agent_name: String,

    // 取得した値
    session_id: String,
    token: String,

    // 接続状態フラグ
    connected: bool,

    // 何かメッセージをUIに表示したいとき
    status_message: String,

    // スライドの情報
    current_slide_index: usize,
    total_slide_count: usize,
}

impl AppState {
    pub fn solve_otp(&mut self) {
        let client = reqwest::Client::new();
        let base_url = self.primary_server_address.clone();
        let otp = self.otp.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(verify_otp(&client, &base_url, &otp));
            let mut state = APP_STATE.lock().unwrap();

            match result {
                Ok(response) => {
                    state.session_id = response.session_id;
                    state.token = response.token;
                    state.status_message = "Otp verified".to_owned();
                }
                Err(e) => {
                    state.status_message = format!("Connection failed: {}", e);
                }
            }
        });
    }

    pub fn establish_ws_connection(&mut self) {
        let session_id = self.session_id.clone();
        let token = self.token.clone();
        let agent_name = self.agent_name.clone();
        let session_server_address = self.session_server_address.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(establish_ws_connection(
                &session_server_address,
                &session_id,
                &token,
                &agent_name,
            ));
            let mut state = APP_STATE.lock().unwrap();
            match result {
                Ok(()) => {
                    state.connected = true;
                    state.status_message = "WS connection established".to_owned();
                }
                Err(e) => {
                    state.status_message = format!("Failed to establish WS connection: {}", e);
                }
            }
        });
    }
}

static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

// ----------------------
// 3) main関数
// ----------------------
fn main() -> eframe::Result {
    // 今のバージョンのeframeでは、ウィンドウサイズを「ViewportBuilder」経由で指定する必要がある
    let builder = egui::ViewportBuilder::default()
        .with_title("My egui App")
        .with_inner_size(egui::vec2(400.0, 300.0));

    let options = eframe::NativeOptions {
        viewport: builder,
        ..Default::default()
    };

    // eframe::run_simple_native(...) でウィンドウ起動 & UIループ開始
    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        // ここで毎フレーム ui_main(ctx) を呼び出す形に分割
        ui_main(ctx);
    })
}

// ----------------------
// 4) UIを描画するための関数
// ----------------------
fn ui_main(ctx: &egui::Context) {
    // まずはロックを取ってアプリの状態を取り出す
    let mut state = APP_STATE.lock().unwrap();

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
                            // 切断処理(例)
                            state.connected = false;
                            state.status_message = "Disconnected".to_owned();
                        }
                    });
                });
            });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add_space(8.0);

        // 接続済みかどうかでUIを分岐
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
                    egui::Grid::new("some_unique_id")
                        .num_columns(2)
                        .show(ui, |ui| {
                            ui.label("Address(Primary):");
                            ui.text_edit_singleline(&mut state.primary_server_address);
                            ui.end_row();

                            ui.label("Address(Session):");
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
                        state.solve_otp();
                        
                    }
                },
            );
        }
    });

    egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // 左端に接続状況を表示
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

            // 右端に状態メッセージを表示
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Message: {}", state.status_message));
            });
        });
    });
}
