use eframe::egui;

fn main() -> eframe::Result {
    let builder = egui::ViewportBuilder::default()
        .with_title("My egui App")
        .with_inner_size(egui::vec2(400.0, 300.0));

    let options = eframe::NativeOptions {
        viewport: builder,
        ..Default::default()
    };

    // アプリケーションの状態
    
    // Server configuration
    let mut server_address = "ws://localhost:8080";

    // Session state
    let mut connect_otp = 0000;
    let mut connected_session_id = "".to_owned();

    let mut slide_name = "".to_owned();
    let mut total_slide_count = 0;
    let mut current_slide_index = 0;

    // Agent configuration
    let mut agent_name = "Agent-001".to_owned();

    // Connection state
    let mut connected = false;

    // GUI state
    let mut status_message = "Idle".to_owned();

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            egui::Frame::default()
                .outer_margin(egui::vec2(0.0, 4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.heading("My egui App");
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Disconnect").clicked() {
                                connected = false; // 切断時のロジック
                                status_message = "Disconnected".to_owned();
                            }
                        });
                    });
                });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.with_layout(
            //     egui::Layout::top_down_justified(egui::Align::Center),
            //     |ui| {
            //     },
            // );

            // 接続前なら接続設定を表示, 接続済みなら
        });

        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // 左端に接続状況を表示 (幅が足りない場合は '...' にする)
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label(format!(
                        "Status: {}",
                        if connected {
                            "Connected"
                        } else {
                            "Not Connected"
                        }
                    ));
                });

                // 右端に状態メッセージを表示
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Message: {status_message}"));
                });
            });
        });
    })
}
