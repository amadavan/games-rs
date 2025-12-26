use clap::Parser;
use games_rs::{
    Game, agents::RandomAgent, agents::monte_carlo_graph::MonteCarloGraph,
    agents::train::TrainableComponent, connect_four::ConnectFour,
};
use indicatif::MultiProgress;

#[derive(clap::ValueEnum, Clone, Debug)]
enum AgentType {
    MCGS,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum GameType {
    ConnectFour,
    UltimateTTT,
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, default_value_t = 1000)]
    num_samples: usize,
    #[clap(long, required = true)]
    game: GameType,
    #[clap(long, required = false)]
    agents: Vec<AgentType>,
    #[clap(long, default_value_t = true)]
    verbose: bool,
}

pub fn main() {
    let args = Args::parse();

    println!("{:?}", args);
    let mpb = if args.verbose {
        let pb = MultiProgress::new();
        Some(pb)
    } else {
        None
    };

    let a1_agent = Box::new(RandomAgent::<ConnectFour>::new());
    let a2_agent = Box::new(RandomAgent::<ConnectFour>::new());

    let _ = games_rs::agents::train::play_batch::<ConnectFour>(
        a1_agent.as_ref(),
        a2_agent.as_ref(),
        args.num_samples,
        mpb.as_ref(),
    );

    let batch = games_rs::agents::train::play_batch_parallel::<ConnectFour, _, _>(
        || Box::new(RandomAgent::<ConnectFour>::new()),
        || Box::new(RandomAgent::<ConnectFour>::new()),
        args.num_samples,
        mpb.as_ref(),
    );

    for agent_type in &args.agents {
        // println!("Training agent: {:?}", agent_type);
        match agent_type {
            AgentType::MCGS => {
                let mut mcgs_agent = MonteCarloGraph::<ConnectFour>::new();

                mcgs_agent.train_batch(&batch, mpb.as_ref());
            }
        }
    }
}
