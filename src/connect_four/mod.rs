pub mod agents;
pub mod board;

pub fn play_game<A1: agents::Agent, A2: agents::Agent>(a1: &A1, a2: &A2) -> board::BoardStatus {
    let mut board = board::Board::new();

    let mut current_player = board::Token::Red;
    let mut moves = Vec::new();

    while board.get_status() == board::BoardStatus::InProgress {
        let mv = match current_player {
            board::Token::Red => a1.get_move(&board, &moves),
            board::Token::Yellow => a2.get_move(&board, &moves),
            board::Token::Empty => panic!("Empty player cannot make a move"),
        };

        let play = board.play(mv, current_player);
        if play.is_err() {
            println!("Invalid move attempted by {:?}: {:?}", current_player, mv);
            continue;
        }

        moves.push((current_player, board.clone()));

        current_player = match current_player {
            board::Token::Red => board::Token::Yellow,
            board::Token::Yellow => board::Token::Red,
            board::Token::Empty => panic!("Empty player cannot make a move"),
        };
    }

    board.get_status().clone()
}
