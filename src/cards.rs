use derive_aliases::derive;
use macros::enum_meta;

#[derive(..StdTraits)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
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

#[derive(..StdTraits)]
pub struct Card {
    suit: Suit,
    rank: Rank,
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

    pub fn new_standard() -> Self {
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

    pub fn new_with_ranks(suits: &[Suit], ranks: &[Rank]) -> Self {
        let mut cards = VecDeque::new();

        for &suit in suits {
            for &rank in ranks {
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
        write!(f, "[ {:?} ]", self.cards)
    }
}
