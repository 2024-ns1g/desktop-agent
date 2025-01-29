use crate::{models::events::WsEvent, APP_STATE};
use eframe::egui;

pub mod state;

pub fn ui_main(ctx: &egui::Context) {
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

    // Update state with pending events
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
                            state.disconnect();
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
