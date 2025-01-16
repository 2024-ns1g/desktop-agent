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
        // ヘッダー
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("My egui Application"); // タイトル
                if ui.button("Disconnect").clicked() {
                    connected = false; // 切断のロジック
                    status_message = "Disconnected".to_owned();
                }
            });
        });

        // メインコンテンツ
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

        // フッター
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let connection_status = if connected { "Connected" } else { "Not Connected" };
                ui.label(format!("Status: {connection_status}"));
                ui.label(format!("Message: {status_message}"));
            });
        });
    })
}
