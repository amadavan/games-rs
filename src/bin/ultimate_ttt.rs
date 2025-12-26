use clap::Parser;
use games_rs::{
    GameStatus, PlayThrough,
    agents::{Agent, MinimaxAgent, PlayerAgent, RandomAgent, scorer::naive_scorer::NaiveScorer},
    ultimate_ttt::UltimateTTT,
};

type G = UltimateTTT;

#[derive(clap::ValueEnum, Clone, Debug)]
enum AvailableAgents {
    Minimax,
    Player,
    Random,
}

impl ToString for AvailableAgents {
    fn to_string(&self) -> String {
        match self {
            AvailableAgents::Minimax => "Minimax".to_string(),
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
    let ai_player1: Box<dyn Agent<G>> = match Args::parse().player1 {
        AvailableAgents::Minimax => Box::new({
            let scorer = NaiveScorer::<G>::new();
            MinimaxAgent::<G, _>::new(4, scorer)
        }),
        AvailableAgents::Player => Box::new(PlayerAgent::<G>::new(1)),
        AvailableAgents::Random => Box::new(RandomAgent::<G>::new()),
    };

    let ai_player2: Box<dyn Agent<G>> = match Args::parse().player2 {
        AvailableAgents::Minimax => Box::new({
            let scorer = NaiveScorer::<G>::new();
            MinimaxAgent::<G, _>::new(4, scorer)
        }),
        AvailableAgents::Player => Box::new(PlayerAgent::<G>::new(2)),
        AvailableAgents::Random => Box::new(RandomAgent::<G>::new()),
    };

    let playthrough = games_rs::play_game::<G>(ai_player1.as_ref(), ai_player2.as_ref());

    match playthrough.get_result() {
        GameStatus::Win(player) => println!("Player {} wins!", player),
        GameStatus::Draw => println!("The game is a draw!"),
        GameStatus::InProgress => println!("The game is still in progress!"),
    }
}
