use connection::{establish_ws_connection, verify_otp};
use eframe::egui;
use once_cell::sync::Lazy;
use std::sync::Mutex;

mod connection;

#[derive(Default)]
struct AppState {
    // User-input parameters
    primary_server_address: String,
    session_server_address: String,

    otp: String,
    agent_name: String,

    // Retrieved values
    session_id: String,
    token: String,

    // Connection status flag
    connected: bool,

    // Status message to display in the UI
    status_message: String,

    // Slide information
    current_slide_index: usize,
    total_slide_count: usize,
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
                    // Establish WebSocket connection after successful OTP verification
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
                    state.status_message = "WebSocket connection established.".to_owned();
                }
                Err(e) => {
                    state.status_message = format!("Failed to establish WebSocket connection: {}", e);
                }
            }
        });
    }
}

static APP_STATE: Lazy<Mutex<AppState>> = Lazy::new(|| Mutex::new(AppState::default()));

fn main() -> eframe::Result {
    env_logger::init();

    let builder = egui::ViewportBuilder::default()
        .with_title("My egui App")
        .with_inner_size(egui::vec2(400.0, 300.0));

    let options = eframe::NativeOptions {
        viewport: builder,
        ..Default::default()
    };

    // Launch the window and start the UI loop
    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        // Call ui_main(ctx) every frame
        ui_main(ctx);
    })
}

fn ui_main(ctx: &egui::Context) {
    // Acquire the lock to access the app state
    let mut state = APP_STATE.lock().unwrap();

    egui::TopBottomPanel::top("header").show(ctx, |ui| {
        egui::Frame::default()
            .outer_margin(egui::vec2(0.0, 4.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Left side
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.heading("My egui App");
                    });

                    // Right side
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Disconnect").clicked() {
                            // Disconnect logic (example)
                            state.connected = false;
                            state.status_message = "Disconnected".to_owned();
                        }
                    });
                });
            });
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add_space(8.0);

        // Branch the UI based on connection status
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
                        // Removed: state.establish_ws_connection();
                    }
                },
            );
        }
    });

    egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Display connection status on the left
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

            // Display status message on the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("Message: {}", state.status_message));
            });
        });
    });
}
