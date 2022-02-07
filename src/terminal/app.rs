use std::error::Error;
use termion::event::Key;
use tui::{Frame, backend::Backend};
use async_trait::async_trait;

use super::InputResult;


#[async_trait]
pub(super) trait App {
    async fn update(&mut self);
    fn draw<B: Backend>(&self, f: &mut Frame<B>);
    async fn handle_input(&mut self, input: Key) -> Result<InputResult, Box<dyn Error>>;
}