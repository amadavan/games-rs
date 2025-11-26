use games_rs::{
    algorithms::monte_carlo_graph::MonteCarloGraph,
    connect_four::{
        agents::{
            Agent, mcgs_agent::MonteCarloGraphSearch, player_agent::PlayerAgent,
            random_agent::RandomAgent,
        },
        board::{BoardStatus, Token},
        play_game,
    },
};

pub fn main() {
    // let mut ai_player1 = PlayerAgent::new(Token::Red);
    let mut ai_player1 = RandomAgent::new(Token::Yellow);

    let mut mcg = MonteCarloGraph::new();
    let mut ai_player2 = MonteCarloGraphSearch::new(Token::Red, &mut mcg);

    for _ in 0..100000 {
        let (result, moves) = play_game(&mut ai_player1, &mut ai_player2);
        ai_player1.notify_win(&moves, result.clone());
        ai_player2.notify_win(&moves, result.clone());

        match result {
            BoardStatus::Win(player) => println!("Player {:?} wins!", player),
            BoardStatus::Draw => println!("The game is a draw!"),
            BoardStatus::InProgress => println!("The game is still in progress!"),
        }
    }
}
