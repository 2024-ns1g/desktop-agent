use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionState {
    #[serde(rename = "currentPage")]
    pub current_page: i32,
    #[serde(rename = "currentStep")]
    pub current_step: i32,
    #[serde(rename = "activeVoteIds")]
    pub active_vote_ids: Vec<String>,
    pub votes: Vec<Vote>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vote {
    #[serde(rename = "voteId")]
    pub vote_id: String,
    #[serde(rename = "choiceId")]
    pub choice_id: String,
    #[serde(rename = "voterId")]
    pub voter_id: String,
}
