mod ecs;
mod game;

use game::Game;

fn main() {
    env_logger::init();
    Game::new().run();
}
