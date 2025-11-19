use games_rs::ultimate_ttt::agents::{Agent, random_agent::RandomAgent};
use games_rs::ultimate_ttt::{Player, play_game};
use games_rs::ultimate_ttt::board::BoardStatus;



fn main() {
    let ai_player1 = RandomAgent::new(Player::X);
    let ai_player2 = RandomAgent::new(Player::O);

    let result = play_game(&ai_player1, &ai_player2);

    match result {
        BoardStatus::Won(player) => println!("Player {:?} wins!", player),
        BoardStatus::Draw => println!("The game is a draw!"),
        BoardStatus::InProgress => println!("The game is still in progress!"),
    }
}