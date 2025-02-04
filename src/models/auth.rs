use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct VerifyOtpRequest {
    pub otp: String,
}

#[derive(Deserialize, Debug)]
pub struct VerifyOtpResponse {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "aggregatorUrl")]
    pub aggregator_url: String,
    pub token: String,
}
