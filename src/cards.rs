use derive_aliases::derive;
use macros::enum_meta;
use std::{collections::VecDeque, fmt::Debug};

#[derive(..StdTraits)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl Suit {
    pub fn symbol(&self) -> char {
        match self {
            Suit::Hearts => '♥',
            Suit::Diamonds => '♦',
            Suit::Clubs => '♣',
            Suit::Spades => '♠',
        }
    }
}

impl Debug for Suit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

#[derive(..StdTraits)]
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
        }
    }
}

impl Debug for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

#[derive(..StdTraits)]
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

impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.rank.symbol(), self.suit.symbol())
    }
}

#[derive(..Eq, ..Ord, Hash)]
pub struct Deck {
    cards: VecDeque<Card>,
}

impl Deck {
    pub fn new_empty() -> Self {
        Deck {
            cards: VecDeque::new(),
        }
    }

    pub fn new() -> Self {
        let mut cards = VecDeque::new();

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
                cards.push_back(Card {
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
        let mut cards_vec: Vec<Card> = self.cards.drain(..).collect();
        cards_vec.reverse();
        self.cards = VecDeque::from(cards_vec);
    }

    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;

        let mut rng = rand::rng();
        let mut cards_vec: Vec<Card> = self.cards.drain(..).collect();
        cards_vec.shuffle(&mut rng);
        self.cards = VecDeque::from(cards_vec);
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop_front()
    }

    pub fn push_top(&mut self, card: Card) {
        self.cards.push_front(card);
    }

    pub fn push_bottom(&mut self, card: Card) {
        self.cards.push_back(card);
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
