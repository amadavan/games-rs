use crate::cards::{Card, Deck};
use derive_aliases::derive;

#[derive(..StdTraits)]
pub enum Action {
    DrawFromDeck,
    DrawFromDiscard,
    Discard(Card),
    Knock(Card),
}

#[derive(..Eq, ..Ord, Hash)]
pub struct Rummy {
    // Fields for the Gin Rummy game
    deck: Deck,
    discard: Deck,
    hands: [Vec<Card>; 2],
    current_player: u8,
}

impl Rummy {
    pub fn new() -> Self {
        let mut deck = Deck::new();
        deck.shuffle();

        Rummy {
            deck,
            discard: Deck::new_empty(),
            hands: [Vec::new(), Vec::new()],
            current_player: 1,
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

    pub fn get_current_player(&self) -> u8 {
        self.current_player
    }

    pub fn get_hand(&self, player: u8) -> Option<&Vec<Card>> {
        match player {
            1 => Some(&self.hands[0]),
            2 => Some(&self.hands[1]),
            _ => None,
        }
    }

    pub fn play_action(&mut self, player: u8, action: Action) -> Result<(), String> {
        if player != self.current_player {
            return Err("Not this player's turn".to_string());
        }

        // Validation step for drawing
        if action == Action::DrawFromDeck || action == Action::DrawFromDiscard {
            if self.get_hand(player).unwrap().len() >= 11 {
                return Err("Cannot draw more cards, hand is full".to_string());
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

    pub fn draw_card(&mut self, player: u8, from_discard: bool) -> Result<(), String> {
        let card = if from_discard {
            self.discard.draw()
        } else {
            self.deck.draw()
        };

        match card {
            Some(c) => {
                match player {
                    1 => self.hands[0].push(c),
                    2 => self.hands[1].push(c),
                    _ => return Err("Invalid player".to_string()),
                }
                Ok(())
            }
            None => Err("No cards left to draw".to_string()),
        }
    }

    pub fn discard_card(&mut self, player: u8, card: Card) -> Result<(), String> {
        let hand = match player {
            1 => &mut self.hands[0],
            2 => &mut self.hands[1],
            _ => return Err("Invalid player".to_string()),
        };

        if let Some(pos) = hand.iter().position(|x| *x == card) {
            hand.remove(pos);
            self.discard.push_top(card);
            self.current_player = if self.current_player == 1 { 2 } else { 1 };
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
        moves = self.get_hand(self.current_player).unwrap().iter().map(| c | Action::Discard(*c)).collect();

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

