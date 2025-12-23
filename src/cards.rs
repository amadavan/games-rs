//! Standard playing card types and deck implementation.
//!
//! This module provides representations for playing cards, including suits, ranks,
//! individual cards, and a deck with common operations like shuffling and drawing.
//!
//! # Examples
//!
//! ```
//! use games_rs::cards::{Card, Deck, Suit, Rank};
//!
//! // Create a new standard 52-card deck
//! let mut deck = Deck::new();
//! deck.shuffle();
//!
//! // Draw cards from the deck
//! let card = deck.draw().unwrap();
//! println!("Drew: {:?}", card); // e.g., "Drew: A♠"
//!
//! // Create individual cards
//! let ace_of_spades = Card::new(Suit::Spades, Rank::Ace);
//! ```

use derive_aliases::derive;
use macros::enum_meta;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt::Debug, str::FromStr};
use tinyvec::ArrayVec;

/// Represents the suit of a playing card.
///
/// # Examples
///
/// ```
/// use games_rs::cards::Suit;
///
/// let suit = Suit::Hearts;
/// assert_eq!(suit.symbol(), '♥');
/// ```
#[derive(..StdTraits, Serialize, Deserialize)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
    Joker,
}

impl Suit {
    /// Returns the Unicode symbol representing this suit.
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

/// Represents the rank of a playing card.
///
/// # Examples
///
/// ```
/// use games_rs::cards::Rank;
///
/// let rank = Rank::Ace;
/// assert_eq!(rank.symbol(), 'A');
/// let value: u8 = rank.into();
/// assert_eq!(value, 14);
/// ```
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
    /// Returns the single-character symbol representing this rank.
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

impl Into<u8> for Rank {
    fn into(self) -> u8 {
        match self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 11,
            Rank::Queen => 12,
            Rank::King => 13,
            Rank::Ace => 14,
            Rank::Joker => 0,
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

/// A playing card with a suit and rank.
///
/// # Examples
///
/// ```
/// use games_rs::cards::{Card, Suit, Rank};
///
/// let card = Card::new(Suit::Spades, Rank::Ace);
/// assert_eq!(card.suit(), &Suit::Spades);
/// assert_eq!(card.rank(), &Rank::Ace);
/// println!("{:?}", card); // Prints "A♠"
/// ```
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

/// A deck of playing cards with operations for shuffling, drawing, and manipulation.
///
/// # Examples
///
/// ```
/// use games_rs::cards::{Deck, Card, Suit, Rank};
///
/// // Create and shuffle a standard deck
/// let mut deck = Deck::new();
/// assert_eq!(deck.len(), 52);
/// deck.shuffle();
///
/// // Draw cards
/// let card1 = deck.draw().unwrap();
/// let card2 = deck.draw().unwrap();
/// assert_eq!(deck.len(), 50);
///
/// // Push cards back
/// deck.push_top(card1);
/// deck.push_bottom(card2);
/// ```
#[derive(..StdTraits, Serialize, Deserialize)]
pub struct Deck {
    // cards: Array<Card, 52>,
    cards: ArrayVec<[Card; 52]>,
}

impl Deck {
    /// Creates an empty deck with no cards.
    pub fn new_empty() -> Self {
        Deck {
            cards: ArrayVec::new(),
        }
    }

    /// Creates a new standard 52-card deck in a fixed order.
    pub fn new() -> Self {
        let mut cards = ArrayVec::<[Card; 52]>::new();
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
        let mut cards_vec: ArrayVec<[Card; 52]> = self.cards.drain(..).collect();
        cards_vec.reverse();
        self.cards = ArrayVec::from(cards_vec);
    }

    /// Randomly shuffles the deck using a cryptographically secure RNG.
    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;

        let mut rng = rand::rng();
        let mut cards_vec: ArrayVec<[Card; 52]> = self.cards.drain(..).collect();
        cards_vec.shuffle(&mut rng);
        self.cards = ArrayVec::from(cards_vec);
    }

    /// Draws a card from the top of the deck, returning `None` if the deck is empty.
    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    /// Adds a card to the top of the deck (next card to be drawn).
    pub fn push_top(&mut self, card: Card) {
        self.cards.push(card);
    }

    /// Adds a card to the bottom of the deck (last card to be drawn).
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
