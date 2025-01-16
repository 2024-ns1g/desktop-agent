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
    let mut name = "Arthur".to_owned();
    let mut age = 42;
    let mut connected = false; // 接続状況のフラグ
    let mut status_message = "Idle".to_owned(); // 状態メッセージ

    eframe::run_simple_native("My egui App", options, move |ctx, _frame| {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            egui::Frame::default()
                .outer_margin(egui::vec2(0.0, 10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.label("My egui App");
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
            ui.heading("Main Content");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                age += 1;
            }
            ui.label(format!("Hello '{name}', age {age}"));
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
