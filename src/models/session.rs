use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfoPageScript {
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfoPage {
    #[serde(rename = "pageId")]
    pub page_id: String,
    pub title: String,
    pub scripts: Vec<SessionInfoPageScript>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfoAvailableVoteChoice {
    #[serde(rename = "choiceId")]
    pub choice_id: String,
    pub title: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfoAvailableVote {
    #[serde(rename = "voteId")]
    pub vote_id: String,
    pub title: String,
    pub description: Option<String>,
    pub choices: Vec<SessionInfoAvailableVoteChoice>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfoVote {
    #[serde(rename = "voteId")]
    pub vote_id: String,
    #[serde(rename = "choiceId")]
    pub choice_id: String,
    #[serde(rename = "voterId")]
    pub voter_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfoState {
    #[serde(rename = "currentPage")]
    pub current_page: i8,
    #[serde(rename = "currentVoteId")]
    pub available_vote_id: Option<String>,
    pub votes: Vec<SessionInfoVote>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionInfo {
    #[serde(rename = "sessionId")]
    pub session_id: String,
    #[serde(rename = "slideId")]
    pub slide_id: String,
    pub title: String,
    pub pages: Vec<SessionInfoPage>,
    #[serde(rename = "availableVotes")]
    pub available_votes: Vec<SessionInfoAvailableVote>,
    pub state: SessionInfoState,
}
