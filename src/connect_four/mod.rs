pub mod board;
pub mod agents;


pub fn play_game<A1: agents::Agent, A2: agents::Agent>(a1: &A1, a2: &A2) -> board::BoardStatus {
    let mut board = board::Board::new();

    let mut current_player = board::Token::Red;

    while board.get_status() == board::BoardStatus::InProgress {
        let mv = match current_player {
            board::Token::Red => a1.get_move(&board),
            board::Token::Yellow => a2.get_move(&board),
            board::Token::Empty => panic!("Empty player cannot make a move"),
        };

        let play = board.play(mv, current_player);
        if play.is_err() {
            println!("Invalid move attempted by {:?}: {:?}", current_player, mv);
            continue;
        }

        current_player = match current_player {
            board::Token::Red => board::Token::Yellow,
            board::Token::Yellow => board::Token::Red,
            board::Token::Empty => panic!("Empty player cannot make a move"),
        };
    }

    board.get_status().clone()
}
