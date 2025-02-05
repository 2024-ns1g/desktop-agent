use crate::api::auth::verify_otp;
use crate::api::session::get_session_info;
use crate::api::state::get_session_state;
use crate::models::events::WsEvent;
use crate::models::session::SessionInfoPage;
use crate::websocket::{run_websocket, WsHandle};
use crate::APP_STATE;

#[derive(Default)]
pub struct AppState {
    pub primary_server_address: String,
    pub session_server_address: String,
    pub otp: String,
    pub agent_name: String,
    pub session_id: String,
    pub token: String,
    pub connected: bool,
    pub status_message: String,
    pub slide_name: String,
    pub current_slide_index: usize,
    pub total_slide_count: usize,
    pub current_step: usize,
    pub pages: Vec<SessionInfoPage>,
    pub ws_event_receiver: Option<std::sync::mpsc::Receiver<WsEvent>>,
    pub logs: Vec<String>,
    pub ws_handle: Option<WsHandle>,
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
                        state.session_server_address = response.aggregator_url;
                        state.status_message = "OTP verified successfully.".to_owned();
                    }
                    APP_STATE.lock().unwrap().fetch_session_info();
                    APP_STATE.lock().unwrap().fetch_session_state();
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
                    Ok(_) => "WebSocket connection closed.".to_owned(),
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
                    // state.current_slide_index = response.state.current_page as usize;
                    // Workdaround
                    state.current_slide_index = 0;
                    state.total_slide_count = response.pages.len();
                    state.pages = response.pages;
                }
                Err(e) => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.status_message = format!("Failed to fetch session info: {}", e);
                }
            }
        });
    }

    pub fn fetch_session_state(&mut self) {
        let client = reqwest::Client::new();
        let base_url = self.session_server_address.clone();
        let session_id = self.session_id.clone();
        let token = self.token.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(get_session_state(&client, &base_url, &session_id, &token));
            match result {
                Ok(response) => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.current_slide_index = response.current_page as usize;
                    state.current_step = response.current_step as usize;
                }
                Err(e) => {
                    let mut state = APP_STATE.lock().unwrap();
                    state.status_message = format!("Failed to fetch session state: {}", e);
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
