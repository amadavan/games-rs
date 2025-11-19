use games_rs::connect_four::{
    agents::{player_agent::PlayerAgent, random_agent::RandomAgent},
    board::{Board, BoardStatus, Token},
    play_game,
};

pub fn main() {
    let ai_player1 = PlayerAgent::new(Token::Red);
    let ai_player2 = RandomAgent::new(Token::Yellow);

    let result = play_game(&ai_player1, &ai_player2);

    match result {
        BoardStatus::Win(player) => println!("Player {:?} wins!", player),
        BoardStatus::Draw => println!("The game is a draw!"),
        BoardStatus::InProgress => println!("The game is still in progress!"),
    }
}
