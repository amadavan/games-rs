use games_rs::connect_four::board::{Token, Board};



pub fn main() {
    let mut board = Board::new();
    board.play(3, Token::Red).unwrap();
    board.play(4, Token::Yellow).unwrap();
    println!("{}", board);
}