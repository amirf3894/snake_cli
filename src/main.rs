use snake::game::snake;

#[tokio::main]
async fn main() {
    snake::main_snake().await.unwrap();
}
