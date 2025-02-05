use crate::{models::events::Event, APP_STATE};
use eframe::egui::FontData;
use egui::FontFamily;

pub mod state;

pub fn ui_main(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "Nyashi".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/Nyashi.ttf")).into(),
    );

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "Nyashi".to_owned());

    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .insert(0, "Nyashi".to_owned());

    ctx.set_fonts(fonts);

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
                Event::ConnectionEstablished => {
                    state.connected = true;
                    state.status_message = "WebSocket connected".to_owned();
                }
                Event::SlideChanged { new_page_index } => {
                    state.current_slide_index = new_page_index;
                    // Add log
                    state.logs.push(format!("ページを{}に変更しました", new_page_index));
                }
                Event::StepChanged {
                    new_page_index,
                    new_step_index,
                } => {
                    state.current_slide_index = new_page_index;
                    state.current_step = new_step_index;
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
