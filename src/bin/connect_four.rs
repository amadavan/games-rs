use clap::Parser;
use games_rs::{
    agents::{Agent, MinimaxAgent, PlayerAgent, RandomAgent, scorer::naive_scorer::NaiveScorer},
    connect_four::ConnectFour,
};

type Game = ConnectFour;

#[derive(clap::ValueEnum, Clone, Debug)]
enum AvailableAgents {
    Minimax,
    MCGS,
    Player,
    Random,
}
impl ToString for AvailableAgents {
    fn to_string(&self) -> String {
        match self {
            AvailableAgents::Minimax => "Minimax".to_string(),
            AvailableAgents::MCGS => "MCGS".to_string(),
            AvailableAgents::Player => "Player".to_string(),
            AvailableAgents::Random => "Random".to_string(),
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, default_value_t = AvailableAgents::Player)]
    player1: AvailableAgents,
    #[clap(long, default_value_t = AvailableAgents::Minimax)]
    player2: AvailableAgents,
}

fn main() {
    let ai_player1: Box<dyn Agent<Game>> = match Args::parse().player1 {
        AvailableAgents::Minimax => Box::new({
            let scorer = NaiveScorer::<Game>::new();
            MinimaxAgent::<Game, _>::new(4, scorer)
        }),
        AvailableAgents::MCGS => Box::new(games_rs::agents::MonteCarloGraphSearch::<Game>::new()),
        AvailableAgents::Player => Box::new(PlayerAgent::<Game>::new(1)),
        AvailableAgents::Random => Box::new(RandomAgent::<Game>::new()),
    };

    let ai_player2: Box<dyn Agent<Game>> = match Args::parse().player2 {
        AvailableAgents::Minimax => Box::new({
            let scorer = NaiveScorer::<Game>::new();
            MinimaxAgent::<Game, _>::new(4, scorer)
        }),
        AvailableAgents::MCGS => Box::new(games_rs::agents::MonteCarloGraphSearch::<Game>::new()),
        AvailableAgents::Player => Box::new(PlayerAgent::<Game>::new(2)),
        AvailableAgents::Random => Box::new(RandomAgent::<Game>::new()),
    };

    let (result, _) = games_rs::play_game::<Game>(ai_player1.as_ref(), ai_player2.as_ref());

    match result {
        games_rs::BoardStatus::Win(player) => println!("Player {} wins!", player),
        games_rs::BoardStatus::Draw => println!("The game is a draw!"),
        games_rs::BoardStatus::InProgress => println!("The game is still in progress!"),
    }
}
