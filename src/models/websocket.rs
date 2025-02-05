use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct RegisterAgentMessageData<'a> {
    #[serde(rename = "agentName")]
    pub agent_name: &'a str,
    #[serde(rename = "agentType")]
    pub agent_type: &'a str,
    pub token: &'a str,
}

#[derive(Serialize)]
pub struct RegisterAgentMessage<'a> {
    #[serde(rename = "requestType")]
    pub msg_type: &'a str,
    pub data: RegisterAgentMessageData<'a>,
}

// #[derive(Deserialize)]
// #[serde(tag = "type")]
// pub enum Event {
//     #[serde(rename = "KEY_PRESS")]
//     KeyPress { key: String },
//     #[serde(rename = "SLIDE_CHANGED")]
//     SlideChanged {
//         slide_index: usize,
//         total_slides: usize,
//     },
// }

#[derive(Deserialize)]
pub struct ChangeCurrentPageData {
    #[serde(rename = "newPageIndex")]
    pub new_page_index: usize,
}

#[derive(Deserialize)]
#[serde(tag = "requestType")]
pub enum Event {
    #[serde(rename = "CHANGE_CURRENT_PAGE")]
    ChangeCurrentPage {
        data: ChangeCurrentPageData,
    },
}
