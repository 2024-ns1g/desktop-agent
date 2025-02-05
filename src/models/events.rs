#[derive(Debug)]
pub enum Event {
    ConnectionEstablished,
    SlideChanged { new_page_index: usize },
}
