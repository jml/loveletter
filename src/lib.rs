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


struct Deck([Card, ..CARDS_IN_DECK]);

#[deriving(Show, PartialEq, Eq)]
enum DeckError {
    WrongCards,
    WrongNumber(uint),
}

impl Deck {
    /// Returns a new, shuffled deck.
    fn new() -> Deck {
        Deck(DECK).shuffled()
    }

    fn from_array(cards: [Card, ..CARDS_IN_DECK]) -> Result<Deck, DeckError> {
        if is_valid_deck(cards.as_slice()) {
            Ok(Deck(cards))
        } else {
            Err(WrongCards)
        }
    }

    fn from_slice(cards: &[Card]) -> Result<Deck, DeckError> {
        if cards.len() != CARDS_IN_DECK {
            return Err(WrongNumber(cards.len()));
        }
        // XXX: Is there a way I can do this without needing to explicitly
        // mention any element?
        let card_array: [Card, ..CARDS_IN_DECK] = [
            cards[0],
            cards[1],
            cards[2],
            cards[3],
            cards[4],
            cards[5],
            cards[6],
            cards[7],
            cards[8],
            cards[9],
            cards[10],
            cards[11],
            cards[12],
            cards[13],
            cards[14],
            cards[15],
            ];
        Deck::from_array(card_array)
    }

    fn shuffled(&self) -> Deck {
        let &Deck(mut cards) = self;
        let mut rng = rand::task_rng();
        rng.shuffle(cards);
        Deck(cards)
    }
}

fn is_valid_deck(deck: &[Card]) -> bool {
    // XXX: Probably don't need to collect & sort the full deck here, since
    // it's already sorted. Not sure if there's a way to assert that at
    // compile time.
    let mut full_deck: Vec<&Card> = DECK.iter().collect();
    let mut sorted_deck: Vec<&Card> = deck.iter().collect();
    full_deck.sort();
    sorted_deck.sort();
    full_deck == sorted_deck
}

// Game state:
// - discarded ('burnt') card
// - the remaining deck
// - each player's card
// - whether they are protected by priestess
// - each player's discard
//   - publicly available

// XXX: Should we wrap up 'Player'?

struct Game {
    _hands: [Card, ..NUM_PLAYERS],
    _burned: Card,
    _deck: Vec<Card>,
}


impl Game {
    fn new() -> Game {
        Game::from_deck(Deck::new())
    }

    fn from_deck(deck: Deck) -> Game {
        let Deck(cards) = deck;
        // XXX: Is there a better syntax for this? (Problem is duplicating
        // NUM_PLAYERS implicitly by explicitly listing cards.)
        let burn = cards[0];
        let hands: [Card, ..NUM_PLAYERS] = [
            cards[1],
            cards[2],
            cards[3],
            cards[4],
            ];
        Game {
            _hands: hands,
            _burned: burn,
            _deck: cards.slice_from(5).iter().map(|&x| x).collect(),
        }
    }

    fn burned_card(&self) -> Card {
        self._burned
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

    //fn handle_turn(&self, |Game, Card| -> Action) -> Game {
        // TODO: UNTESTED:
        // - pop a card off the deck
        // - give it & this to the callback
        // - process the action
        // - update the current player's hand to have whichever card they
        //   didn't discard
        // - increment the internal player count if necessary
    //}
}


fn minister_bust(a: Card, b: Card) -> bool {
    a == Minister && b >= Wizard || a >= Wizard && b == Minister
}


// XXX: Want to have a simple, pure function that knows all of the rules and
// assumes as little as it can. Still not sure the best way to do that.
// Kind of getting blocked on details:
// - what should it return?
// - should the rules function be responsible for things like updating hands,
//   discard piles, etc.
// - how to represent
//   - 'protected by priestess'
//   - 'kicked out of game'
// - how to make sure only allowable actions are played
//   - don't play actions for cards you don't have
//   - soldier
//     - don't allow soldier as guess
//   - for soldier, clown, knight, wizard, general
//     - don't allow self as target
//
// Current best guess at signature:
//   fn judge(current: GameState, dealt_card: Card, action: Action) -> GameState
//
// Where 'Action' combines card & parameters (target player, guess)

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
fn test_all_cards_in_game() {
    // make a new game, make sure that the number & kinds of cards matches the
    // rules (5 soldiers, 2 clowns, etc.)
    let g = Game::new();
    let mut full_deck: Vec<&Card> = DECK.iter().collect();
    full_deck.sort();
    let mut found_cards: Vec<&Card> = g.deck().iter().collect();
    let burnt = g.burned_card();
    found_cards.push(&burnt);
    for card in g.hands().iter() {
        found_cards.push(card);
    }
    found_cards.sort();
    assert_eq!(full_deck, found_cards);
}

#[test]
fn test_from_deck() {
    let cards = [
        Soldier,
        Clown,
        Knight,
        Priestess,
        Wizard,
        General,
        Minister,
        Princess,
        Soldier,
        Clown,
        Soldier,
        Knight,
        Soldier,
        Priestess,
        Soldier,
        Wizard,
        ];
    let deck = Deck(cards);
    let g = Game::from_deck(deck);
    assert_eq!(cards[0], g.burned_card());
    assert_eq!(cards.slice(1, 5), g.hands());
    assert_eq!(cards.slice_from(5), g.deck());
}

#[test]
fn test_minister_bust() {
    assert!(!minister_bust(Soldier, Soldier));
    assert!(minister_bust(Minister, Wizard));
    assert!(minister_bust(Minister, General));
    assert!(minister_bust(Minister, Princess));
    assert!(minister_bust(Wizard, Minister));
    assert!(minister_bust(General, Minister));
    assert!(minister_bust(Princess, Minister));
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
    let Deck(cards) = deck;
    let old_cards = cards.clone();
    deck.shuffled();
    let Deck(new_cards) = deck;
    assert_eq!(old_cards.as_slice(), new_cards.as_slice());
}

#[test]
fn test_deck_fixed_good() {
    match Deck::from_array(DECK) {
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
    match Deck::from_array(cards) {
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
