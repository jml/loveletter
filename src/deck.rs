use std::rand;
use std::rand::Rng;
use std::slice;

use util;

#[deriving(PartialEq, PartialOrd, Eq, Ord, Show, Clone)]
pub enum Card {
    Soldier,
    Clown,
    Knight,
    Priestess,
    Wizard,
    General,
    Minister,
    Princess,
}


static CARDS_IN_DECK: uint = 16;

static DECK: [Card, ..CARDS_IN_DECK] = [
    Soldier,
    Soldier,
    Soldier,
    Soldier,
    Soldier,
    Clown,
    Clown,
    Knight,
    Knight,
    Priestess,
    Priestess,
    Wizard,
    Wizard,
    General,
    Minister,
    Princess,
    ];

#[deriving(Show)]
pub struct Deck(Vec<Card>);

#[deriving(Show, PartialEq, Eq)]
pub enum DeckError {
    WrongCards,
    WrongNumber(uint),
}


impl Deck {
    /// Returns a new, shuffled deck.
    pub fn new() -> Deck {
        // Safe to unwrap because we know that DECK has CARDS_IN_DECK
        // elements.
        Deck::from_slice(&DECK).unwrap().shuffled()
    }

    pub fn from_slice(cards: &[Card]) -> Result<Deck, DeckError> {
        if cards.len() != CARDS_IN_DECK {
            return Err(WrongNumber(cards.len()));
        } else if is_valid_deck(cards) {
            Ok(Deck(cards.iter().map(|x| *x).collect()))
        } else {
            Err(WrongCards)
        }
    }

    pub fn shuffled(&self) -> Deck {
        let &Deck(ref cards) = self;
        let mut new_cards = cards.clone();
        let mut rng = rand::task_rng();
        rng.shuffle(new_cards.as_mut_slice());
        Deck(new_cards)
    }

    pub fn as_slice(&self) -> &[Card] {
        let &Deck(ref cards) = self;
        cards.as_slice()
    }

    pub fn iter(&self) -> slice::Items<Card> {
        let &Deck(ref cards) = self;
        cards.iter()
    }
}

fn is_valid_deck(deck: &[Card]) -> bool {
    let mut full_deck: Vec<&Card> = DECK.iter().collect();
    let mut sorted_deck: Vec<&Card> = deck.iter().collect();
    full_deck.sort();
    sorted_deck.sort();
    full_deck == sorted_deck
}

pub fn is_valid_subdeck(cards: &[Card]) -> bool {
    util::subtract_vector(DECK.iter().map(|&x| x).collect(), cards).is_some()
}


#[test]
fn test_deck_new() {
    let Deck(mut cards) = Deck::new();
    cards.sort();
    assert_eq!(DECK.as_slice(), cards.as_slice());
}

#[test]
fn test_deck_shuffle() {
    let deck = Deck::new();
    let Deck(mut shuffled_cards) = deck.shuffled();
    let Deck(mut cards) = deck;
    cards.sort();
    shuffled_cards.sort();
    assert_eq!(cards.as_slice(), shuffled_cards.as_slice());
}

#[test]
fn test_deck_shuffle_does_not_modify() {
    let deck = Deck::new();
    let Deck(ref cards) = deck;
    let old_cards = cards.clone();
    deck.shuffled();
    let Deck(ref new_cards) = deck;
    assert_eq!(old_cards.as_slice(), new_cards.as_slice());
}

#[test]
fn test_deck_fixed_good() {
    match Deck::from_slice(DECK.as_slice()) {
        Ok(Deck(cards)) => assert_eq!(cards.as_slice(), DECK.as_slice()),
        Err(e) => fail!("Unexpected error: {}", e),
    }
}

#[test]
fn test_deck_fixed_too_many_soldiers() {
    let cards = [
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        ];
    match Deck::from_slice(cards) {
        Ok(Deck(cards)) => fail!("Should not have been OK: {}", cards.as_slice()),
        Err(error) => assert_eq!(error, WrongCards),
    }
}

#[test]
fn test_deck_variable_good() {
    match Deck::from_slice(DECK.as_slice()) {
        Ok(Deck(cards)) => assert_eq!(cards.as_slice(), DECK.as_slice()),
        Err(e) => fail!("Unexpected error: {}", e),
    }
}

#[test]
fn test_deck_variable_too_few() {
    let cards = [Soldier];
    match Deck::from_slice(cards.as_slice()) {
        Ok(Deck(cards)) => fail!("Should not have been OK: {}", cards.as_slice()),
        Err(error) => assert_eq!(error, WrongNumber(cards.len())),
    }
}

#[test]
fn test_deck_variable_too_many() {
    // One soldier too many
    let cards = [
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Soldier,
        Clown,
        Clown,
        Knight,
        Knight,
        Priestess,
        Priestess,
        Wizard,
        Wizard,
        General,
        Minister,
        Princess,
        ];
    match Deck::from_slice(cards.as_slice()) {
        Ok(Deck(cards)) => fail!("Should not have been OK: {}", cards.as_slice()),
        Err(error) => assert_eq!(error, WrongNumber(cards.len())),
    }
}

#[test]
fn test_deck_iter() {
    let deck = Deck::new();
    let i = deck.iter();
    let new_cards: Vec<Card> = i.map(|x| *x).collect();
    let Deck(ref cards) = deck;
    assert_eq!(*cards, new_cards);
}
