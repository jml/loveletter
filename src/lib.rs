use std::collections::TreeSet;
use std::rand;
use std::rand::Rng;

// XXX: Make a loveletter namespace (figure out how to do this properly).

#[deriving(PartialEq, PartialOrd, Eq, Ord, Show)]
enum Card {
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

// Love Letter is a game for exactly four players.
static NUM_PLAYERS: uint = 4;

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

// Game state:
// - discarded card
// - the remaining deck
// - each player's card
// - whether they are protected by priestess
// - each player's discard
//   - publicly available


struct Game {
    _hands: [Card, ..NUM_PLAYERS],
    _burnt: Card,
    _deck: Vec<Card>,
}

// XXX: Next time, remind me not to take the alcohol when they offer it.

impl Game {
    fn new() -> Game {
        let mut cards = DECK;
        let mut rng = rand::task_rng();
        rng.shuffle(cards);
        // XXX: Is there a better syntax for this? (Problem is duplicating
        // NUM_PLAYERS implicitly by explicitly listing cards.)
        let hands: [Card, ..NUM_PLAYERS] = [
            cards[0],
            cards[1],
            cards[2],
            cards[3],
            ];
        let burn = cards[NUM_PLAYERS];
        // XXX: How do we say that we're not going to mutate a variable any more?
        Game {
            _hands: hands,
            _burnt: burn,
            _deck: cards.slice_from(5).iter().map(|&x| x).collect(),
        }
    }

    fn burn_card(&self) -> Card {
        self._burnt
    }

    fn hands(&self) -> &[Card] {
        self._hands
    }

    fn deck(&self) -> &[Card] {
        self._deck.as_slice()
    }

    fn num_cards_remaining(&self) -> uint {
        self._deck.len()
    }
}


#[test]
fn test_card_ordering() {
    assert!(Soldier <= Soldier);
    assert!(Soldier < Clown);
    assert!(Soldier < Princess);
}


#[test]
fn test_new_game() {
    // make a new game, make sure that the number & kinds of cards matches the
    // rules (5 soldiers, 2 clowns, etc.)
    let g = Game::new();
    assert_eq!(11, g.num_cards_remaining())
}

#[test]
fn test_cards_remaining() {
    // XXX: INVARIANT: It is always the case that the remaining deck is a
    // subset of the total deck.
    let g = Game::new();
    let full_deck: TreeSet<Card> = DECK.iter().map(|&x| x).collect();
    let deck: TreeSet<Card> = g.deck().iter().map(|&x| x).collect();
    assert!(full_deck.is_superset(&deck));
}

#[test]
fn test_all_cards_in_game() {
    let g = Game::new();
    let mut full_deck: Vec<&Card> = DECK.iter().collect();
    full_deck.sort();
    let mut found_cards: Vec<&Card> = g.deck().iter().collect();
    let burnt = g.burn_card();
    found_cards.push(&burnt);
    for card in g.hands().iter() {
        found_cards.push(card);
    }
    // XXX: You don't want to test set equality, because that'll eliminate
    // duplicates.
    found_cards.sort();
    assert_eq!(full_deck, found_cards);
}
