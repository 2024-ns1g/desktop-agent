#[derive(Debug)]
pub enum Event {
    ConnectionEstablished,
    SlideChanged { new_page_index: usize },
    StepChanged { new_page_index: usize, new_step_index: usize },
}
