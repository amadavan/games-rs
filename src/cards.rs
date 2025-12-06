use crate::common::array::Array;
use derive_aliases::derive;
use macros::enum_meta;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt::Debug, str::FromStr};

#[derive(..StdTraits, Serialize, Deserialize)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
    Joker,
}

impl Suit {
    pub fn symbol(&self) -> char {
        match self {
            Suit::Hearts => '♥',
            Suit::Diamonds => '♦',
            Suit::Clubs => '♣',
            Suit::Spades => '♠',
            Suit::Joker => 'J',
        }
    }
}

impl FromStr for Suit {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hearts" => Ok(Suit::Hearts),
            "diamonds" => Ok(Suit::Diamonds),
            "clubs" => Ok(Suit::Clubs),
            "spades" => Ok(Suit::Spades),
            _ => Err("Invalid suit".to_string()),
        }
    }
}

impl Debug for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

#[derive(..StdTraits, Serialize, Deserialize)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
    Joker,
}

impl Rank {
    pub fn symbol(&self) -> char {
        match self {
            Rank::Two => '2',
            Rank::Three => '3',
            Rank::Four => '4',
            Rank::Five => '5',
            Rank::Six => '6',
            Rank::Seven => '7',
            Rank::Eight => '8',
            Rank::Nine => '9',
            Rank::Ten => 'T',
            Rank::Jack => 'J',
            Rank::Queen => 'Q',
            Rank::King => 'K',
            Rank::Ace => 'A',
            Rank::Joker => 'J',
        }
    }
}

impl FromStr for Rank {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "2" => Ok(Rank::Two),
            "3" => Ok(Rank::Three),
            "4" => Ok(Rank::Four),
            "5" => Ok(Rank::Five),
            "6" => Ok(Rank::Six),
            "7" => Ok(Rank::Seven),
            "8" => Ok(Rank::Eight),
            "9" => Ok(Rank::Nine),
            "T" | "10" => Ok(Rank::Ten),
            "J" => Ok(Rank::Jack),
            "Q" => Ok(Rank::Queen),
            "K" => Ok(Rank::King),
            "A" => Ok(Rank::Ace),
            _ => Err("Invalid rank".to_string()),
        }
    }
}

impl Debug for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

#[derive(..StdTraits, Serialize, Deserialize)]
pub struct Card {
    suit: Suit,
    rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Card { suit, rank }
    }

    pub fn suit(&self) -> &Suit {
        &self.suit
    }

    pub fn rank(&self) -> &Rank {
        &self.rank
    }
}

impl Default for Card {
    fn default() -> Self {
        Card {
            suit: Suit::Joker,
            rank: Rank::Joker,
        }
    }
}

impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.rank.symbol(), self.suit.symbol())
    }
}

#[derive(..StdTraits, Serialize, Deserialize)]
pub struct Deck {
    cards: Array<Card, 52>,
}

impl Deck {
    pub fn new_empty() -> Self {
        Deck {
            cards: Array::new(),
        }
    }

    pub fn new() -> Self {
        let mut cards = Array::<Card, 52>::new();

        for &suit in &[Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
            for &rank in &[
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
                Rank::Jack,
                Rank::Queen,
                Rank::King,
                Rank::Ace,
            ] {
                cards.push(Card {
                    suit: suit.clone(),
                    rank: rank.clone(),
                });
            }
        }

        Deck { cards }
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn clear(&mut self) {
        self.cards.clear();
    }

    pub fn reverse(&mut self) {
        let mut cards_vec: Array<Card, 52> = self.cards.drain(..).collect();
        cards_vec.reverse();
        self.cards = Array::from(cards_vec);
    }

    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;

        let mut rng = rand::rng();
        let mut cards_vec: Array<Card, 52> = self.cards.drain(..).collect();
        cards_vec.shuffle(&mut rng);
        self.cards = Array::from(cards_vec);
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn push_top(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn push_bottom(&mut self, card: Card) {
        self.cards.insert(self.cards.len(), card);
    }
}

impl Debug for Deck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[ {} ]",
            self.cards
                .iter()
                .map(|c| format!("{:?}", c))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
