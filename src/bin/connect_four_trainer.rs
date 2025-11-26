use clap::Parser;
use games_rs::algorithms::monte_carlo_graph::MonteCarloGraph;
use games_rs::common::filesystem;
use games_rs::connect_four;

type SamplePath = (
    Vec<connect_four::board::Board>,
    connect_four::board::BoardStatus,
);

fn generate_sample_games(n_samples: usize, verbose: bool) -> Vec<SamplePath> {
    let ai_player1 =
        connect_four::agents::random_agent::RandomAgent::new(connect_four::board::Token::Red);
    let ai_player2 =
        connect_four::agents::random_agent::RandomAgent::new(connect_four::board::Token::Yellow);

    let mut samples = Vec::new();

    let pb = if verbose {
        Some(indicatif::ProgressBar::new(n_samples as u64))
    } else {
        None
    };

    for _ in 0..n_samples {
        if let Some(pb) = &pb {
            pb.inc(1);
        }

        let (result, moves) =
            connect_four::play_game(&mut ai_player1.clone(), &mut ai_player2.clone());
        let actions: Vec<connect_four::board::Board> =
            moves.iter().map(|(_, board)| board.clone()).collect();
        samples.push((actions, result));
    }

    if let Some(pb) = &pb {
        pb.finish_and_clear();
    }

    samples
}

fn train_monte_carlo_graph(
    graph: &mut MonteCarloGraph<connect_four::board::Board>,
    samples: Vec<SamplePath>,
    verbose: bool,
) {
    let pb = if verbose {
        Some(indicatif::ProgressBar::new(samples.len() as u64))
    } else {
        None
    };

    for (actions, _result) in samples {
        if let Some(pb) = &pb {
            pb.inc(1);
        }

        graph.back_propogate(actions, true);
    }

    if let Some(pb) = &pb {
        pb.finish_and_clear();
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, default_value = "10000")]
    n_samples: usize,
    #[clap(short, long, default_value_t = false)]
    force: bool,
    #[clap(short, long, default_value_t = false)]
    append: bool,
    #[clap(short, long, default_value = "connect_four_samples.bin")]
    sample_file: String,
    #[clap(short, long, default_value = "connect_four_mcg.bin")]
    mcg_file_name: String,
    #[clap(short, long, default_value_t = true)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    let sample_path = filesystem::get_data_dir().join(&args.sample_file);
    let mcg_path = filesystem::get_data_dir().join(&args.mcg_file_name);

    // Load existing samples if they exist
    let mut samples = if sample_path.exists() {
        let bytes = std::fs::read(&sample_path).unwrap();
        let (samples, _) = bincode::serde::decode_from_slice::<Vec<SamplePath>, _>(
            &bytes,
            bincode::config::standard(),
        )
        .unwrap();
        samples
    } else {
        Vec::new()
    };

    // Generate more samples if needed
    if args.force || args.append || &args.n_samples > &samples.len() {
        let n_to_generate = if args.force || args.append {
            args.n_samples
        } else {
            args.n_samples - samples.len()
        };
        println!("Generating {} Connect Four sample games...", n_to_generate);
        let new_samples = generate_sample_games(n_to_generate, args.verbose);
        if args.append {
            samples.extend(new_samples);
        } else {
            samples = new_samples;
        }
        let serialized =
            bincode::serde::encode_to_vec(&samples, bincode::config::standard()).unwrap();
        std::fs::write(&sample_path, serialized).unwrap();
    }

    // Load the MCG and train on the samples (back-propogate) all the results
    let mut mcg: MonteCarloGraph<connect_four::board::Board> = if mcg_path.exists() {
        MonteCarloGraph::from_file(mcg_path.to_str().unwrap()).unwrap()
    } else {
        MonteCarloGraph::new()
    };

    println!("Training Monte Carlo Graph on {} samples...", samples.len());
    train_monte_carlo_graph(&mut mcg, samples, args.verbose);
}
