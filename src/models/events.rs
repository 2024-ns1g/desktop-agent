#[derive(Debug)]
pub enum WsEvent {
    SlideChanged { index: usize, total: usize },
    KeyPressed(String),
    ConnectionEstablished,
}
