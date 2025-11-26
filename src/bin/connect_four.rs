use games_rs::{
    algorithms::monte_carlo_graph::MonteCarloGraph,
    common::filesystem,
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
    // Load or create Monte Carlo Graph
    let path = filesystem::get_data_dir().join("connect_four_mcg.bin");
    let path_str = path.as_os_str().to_str().unwrap();
    let mut mcg = if path.exists() {
        MonteCarloGraph::from_file(path_str).unwrap()
    } else {
        MonteCarloGraph::new()
    };

    // Create agents
    let mut ai_player1 = RandomAgent::new(Token::Yellow);
    let mut ai_player2 = MonteCarloGraphSearch::new(Token::Red, &mut mcg);

    // Play the actual game
    let (result, moves) = play_game(&mut ai_player1, &mut ai_player2);
    ai_player1.notify_win(&moves, result.clone());
    ai_player2.notify_win(&moves, result.clone());

    match result {
        BoardStatus::Win(player) => println!("Player {:?} wins!", player),
        BoardStatus::Draw => println!("The game is a draw!"),
        BoardStatus::InProgress => println!("The game is still in progress!"),
    }
}
