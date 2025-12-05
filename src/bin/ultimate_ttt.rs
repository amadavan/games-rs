use games_rs::{
    agents::{MinimaxAgent, PlayerAgent, RandomAgent, scorer::naive_scorer::NaiveScorer},
    ultimate_ttt::UltimateTTT,
};

type Game = UltimateTTT;

fn main() {
    // let ai_player1 = MinimaxAgent::<Game>::new(1);
    let scorer1 = NaiveScorer::<Game>::new();
    let ai_player1 = MinimaxAgent::<Game, _>::new(4, scorer1);
    let scorer2 = NaiveScorer::<Game>::new();
    let ai_player2 = MinimaxAgent::<Game, _>::new(4, scorer2);

    let result = games_rs::play_game::<Game, _, _>(&ai_player1, &ai_player2);

    match result {
        games_rs::BoardStatus::Win(player) => println!("Player {} wins!", player),
        games_rs::BoardStatus::Draw => println!("The game is a draw!"),
        games_rs::BoardStatus::InProgress => println!("The game is still in progress!"),
    }
}
