use std::str::FromStr;

use crate::{
    GameBoard,
    cards::{Card, Deck},
    common::array::Array,
};
use derive_aliases::derive;
use serde::{Deserialize, Serialize};

#[derive(..StdTraits, Debug, Serialize, Deserialize)]
pub enum Player {
    Player1,
    Player2,
}

impl From<u8> for Player {
    fn from(value: u8) -> Self {
        match value {
            1 => Player::Player1,
            2 => Player::Player2,
            _ => panic!("Invalid player number"),
        }
    }
}

impl Into<u8> for Player {
    fn into(self) -> u8 {
        match self {
            Player::Player1 => 1,
            Player::Player2 => 2,
        }
    }
}

#[derive(..StdTraits, Debug, Serialize, Deserialize)]
pub enum Action {
    DrawFromDeck,
    DrawFromDiscard,
    Discard(Card),
    Knock(Card),
}

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draw_deck" => Ok(Action::DrawFromDeck),
            "draw_discard" => Ok(Action::DrawFromDiscard),
            _ if s.starts_with("discard ") => {
                let card_str = s.trim_start_matches("discard ");
                // Assuming card_str is in the format "rank_suit"
                let parts: Vec<&str> = card_str.split('_').collect();
                if parts.len() == 2 {
                    let rank = parts[0].parse().map_err(|_| "Invalid rank".to_string())?;
                    let suit = parts[1].parse().map_err(|_| "Invalid suit".to_string())?;
                    Ok(Action::Discard(Card::new(suit, rank)))
                } else {
                    Err("Invalid discard action format".to_string())
                }
            }
            _ if s.starts_with("knock ") => {
                let card_str = s.trim_start_matches("knock ");
                // Similar parsing logic as above
                unimplemented!()
            }
            _ => Err("Unknown action".to_string()),
        }
    }
}

#[derive(..StdTraits, Debug, Serialize, Deserialize)]
pub struct Rummy {
    // Fields for the Gin Rummy game
    deck: Deck,
    discard: Deck,
    hands: [Hand; 2],
    current_player: Player,
}

impl GameBoard for Rummy {
    type MoveType = Action;
    type PlayerType = Player;

    fn get_status(&self) -> crate::BoardStatus {
        if self.deck.is_empty() && self.discard.is_empty() {
            crate::BoardStatus::Draw
        } else {
            crate::BoardStatus::InProgress
        }
    }

    fn get_current_player(&self) -> Player {
        self.current_player
    }

    fn play(&mut self, action: Self::MoveType, player: Player) -> Result<(), String> {
        self.play_action(player, action)
    }

    fn get_available_moves(&self) -> Vec<Self::MoveType> {
        self.get_available_moves()
    }
}

impl Rummy {
    pub fn new() -> Self {
        let mut deck = Deck::new();
        deck.shuffle();

        Rummy {
            deck,
            discard: Deck::new_empty(),
            hands: [Hand::new(), Hand::new()],
            current_player: Player::Player1,
        }
    }

    pub fn deal(&mut self) {
        self.deck.shuffle();

        // Logic to deal cards to players
        for _ in 0..10 {
            if let Some(card) = self.deck.draw() {
                self.hands[0].push(card);
            }
            if let Some(card) = self.deck.draw() {
                self.hands[1].push(card);
            }
        }
    }

    pub fn get_hand(&self, player: Player) -> Option<&Hand> {
        match player {
            Player::Player1 => Some(&self.hands[0]),
            Player::Player2 => Some(&self.hands[1]),
        }
    }

    pub fn play_action(&mut self, player: Player, action: Action) -> Result<(), String> {
        if player != self.current_player {
            return Err("Not this player's turn".to_string());
        }

        // Validation step for drawing
        if action == Action::DrawFromDeck || action == Action::DrawFromDiscard {
            if self.get_hand(player).unwrap().len() > 10 {
                return Err("Cannot draw more cards, hand is full".to_string());
            }
        } else {
            if self.get_hand(player).unwrap().len() < 10 {
                return Err("Must draw a card before discarding".to_string());
            }
        }

        //

        match action {
            Action::DrawFromDeck => self.draw_card(player, false),
            Action::DrawFromDiscard => self.draw_card(player, true),
            Action::Discard(card) => self.discard_card(player, card),
            Action::Knock(card) => {
                // Logic for knocking
                unimplemented!()
            }
        }
    }

    pub fn draw_card(&mut self, player: Player, from_discard: bool) -> Result<(), String> {
        let card = if from_discard {
            self.discard.draw()
        } else {
            self.deck.draw()
        };

        match card {
            Some(c) => {
                match player {
                    Player::Player1 => self.hands[0].push(c),
                    Player::Player2 => self.hands[1].push(c),
                };
                Ok(())
            }
            None => Err("No cards left to draw".to_string()),
        }
    }

    pub fn discard_card(&mut self, player: Player, card: Card) -> Result<(), String> {
        let hand = match player {
            Player::Player1 => &mut self.hands[0],
            Player::Player2 => &mut self.hands[1],
        };

        if let Some(pos) = hand.iter().position(|x| *x == card) {
            hand.remove(pos);
            self.discard.push_top(card);
            self.current_player = match self.current_player {
                Player::Player1 => Player::Player2,
                Player::Player2 => Player::Player1,
            };
            Ok(())
        } else {
            Err("Card not in hand".to_string())
        }
    }

    pub fn get_available_moves(&self) -> Vec<Action> {
        let mut moves = Vec::new();

        // Drawing options
        if self.get_hand(self.current_player).unwrap().len() < 11 {
            moves.push(Action::DrawFromDeck);
            if !self.discard.is_empty() {
                moves.push(Action::DrawFromDiscard);
            }

            return moves;
        }

        // Options to discard
        moves = self
            .get_hand(self.current_player)
            .unwrap()
            .iter()
            .map(|c| Action::Discard(*c))
            .collect();

        // Options for knocking

        moves
    }

    pub fn get_minimal_points(&self, hand: &Vec<Card>) -> u8 {
        // Construct all possible melds
        // TODO: Isolate by rank for sets
        // TODO: Isolate by suit for runs
        // TODO: Identify melds with more than 3 cards
        // TODO: Use combinatorial approach to find best meld combination

        // Logic to calculate minimal points in hand
        unimplemented!()
    }

    // Additional methods for game logic would go here
}

impl Default for Rummy {
    fn default() -> Self {
        Self::new()
    }
}

pub type Hand = Array<Card, 11>; // Max 11 cards in hand during play
