use games_rs::{
    agents::{PlayerAgent, RandomAgent},
    connect_four::ConnectFour,
    ultimate_ttt::UltimateTTT,
};

type Game = ConnectFour;

fn main() {
    let ai_player1 = PlayerAgent::<Game>::new(1);
    let ai_player2 = RandomAgent::<Game>::new();

    let result =
        games_rs::play_game::<Game, PlayerAgent<Game>, RandomAgent<Game>>(&ai_player1, &ai_player2);

    match result {
        games_rs::BoardStatus::Win(player) => println!("Player {} wins!", player),
        games_rs::BoardStatus::Draw => println!("The game is a draw!"),
        games_rs::BoardStatus::InProgress => println!("The game is still in progress!"),
    }
}
