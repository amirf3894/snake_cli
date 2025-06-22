use snake::game::snake;
use std::io::{Write, stdout};
#[tokio::main]
async fn main() {
    snake::main_snake().await.unwrap();
}
