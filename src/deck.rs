use std::rand;
use std::rand::Rng;
use std::slice;

use util;


#[derive(PartialEq, PartialOrd, Eq, Ord, Show, Clone, Copy)]
/// Love Letter has eight different cards, each of different worth. The Soldier is the lowest and
/// the Princess is the highest.
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


const CARDS_IN_DECK: uint = 16;

/// In the Love Letter deck, there are:
/// - 5 Soldiers
/// - 2 Clowns
/// - 2 Knights
/// - 2 Priestesses
/// - 2 Wizards
/// - 1 General
/// - 1 Minister
/// - 1 Princess
///
/// Altogether, there are 16 cards.
const DECK: [Card; CARDS_IN_DECK] = [
    Card::Soldier,
    Card::Soldier,
    Card::Soldier,
    Card::Soldier,
    Card::Soldier,
    Card::Clown,
    Card::Clown,
    Card::Knight,
    Card::Knight,
    Card::Priestess,
    Card::Priestess,
    Card::Wizard,
    Card::Wizard,
    Card::General,
    Card::Minister,
    Card::Princess,
    ];


#[derive(Show)]
/// A Love Letter deck.
pub struct Deck(Vec<Card>);

#[derive(Show, PartialEq, Eq, Copy)]
/// Returned when we try to construct a malformed Love Letter deck.
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

    /// Construct a deck from the given cards. The cards must represent a complete deck.
    pub fn from_slice(cards: &[Card]) -> Result<Deck, DeckError> {
        if cards.len() != CARDS_IN_DECK {
            return Err(DeckError::WrongNumber(cards.len()));
        } else if is_valid_deck(cards) {
            Ok(Deck(cards.iter().map(|x| *x).collect()))
        } else {
            Err(DeckError::WrongCards)
        }
    }

    /// Return a shuffled version of this deck.
    pub fn shuffled(&self) -> Deck {
        let &Deck(ref cards) = self;
        let mut new_cards = cards.clone();
        let mut rng = rand::thread_rng();
        rng.shuffle(new_cards.as_mut_slice());
        Deck(new_cards)
    }

    pub fn as_slice(&self) -> &[Card] {
        let &Deck(ref cards) = self;
        cards.as_slice()
    }

    pub fn iter(&self) -> slice::Iter<Card> {
        let &Deck(ref cards) = self;
        cards.iter()
    }
}

/// Does the given list of cards represent a valid deck? That is, are cards present, with no extra
/// cards.
fn is_valid_deck(deck: &[Card]) -> bool {
    let mut full_deck: Vec<Card> = Vec::new();
    full_deck.push_all(&DECK);
    let mut sorted_deck: Vec<Card> = Vec::new();
    sorted_deck.push_all(deck);
    full_deck.sort();
    sorted_deck.sort();
    full_deck == sorted_deck
}

/// Does the given list of cards represent a valid sub-deck? That is, could we add cards to this
/// list to make up a full deck?
pub fn is_valid_subdeck(cards: &[Card]) -> bool {
    util::subtract_vector(DECK.iter().map(|&x| x).collect(), cards).is_some()
}

#[cfg(test)]
mod test {
    use super::{Card, DECK, Deck, DeckError};

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
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_deck_fixed_too_many_soldiers() {
        let cards = [
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            ];
        match Deck::from_slice(&cards) {
            Ok(Deck(cards)) => panic!("Should not have been OK: {}", cards.as_slice()),
            Err(error) => assert_eq!(error, DeckError::WrongCards),
        }
    }

    #[test]
    fn test_deck_variable_good() {
        match Deck::from_slice(DECK.as_slice()) {
            Ok(Deck(cards)) => assert_eq!(cards.as_slice(), DECK.as_slice()),
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_deck_variable_too_few() {
        let cards = [Card::Soldier];
        match Deck::from_slice(cards.as_slice()) {
            Ok(Deck(cards)) => panic!("Should not have been OK: {}", cards.as_slice()),
            Err(error) => assert_eq!(error, DeckError::WrongNumber(cards.len())),
        }
    }

    #[test]
    fn test_deck_variable_too_many() {
        // One soldier too many
        let cards = [
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Soldier,
            Card::Clown,
            Card::Clown,
            Card::Knight,
            Card::Knight,
            Card::Priestess,
            Card::Priestess,
            Card::Wizard,
            Card::Wizard,
            Card::General,
            Card::Minister,
            Card::Princess,
            ];
        match Deck::from_slice(cards.as_slice()) {
            Ok(Deck(cards)) => panic!("Should not have been OK: {}", cards.as_slice()),
            Err(error) => assert_eq!(error, DeckError::WrongNumber(cards.len())),
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

    #[test]
    fn test_card_ordering() {
        assert!(Card::Soldier <= Card::Soldier);
        assert!(Card::Soldier < Card::Clown);
        assert!(Card::Soldier < Card::Princess);
    }
}
