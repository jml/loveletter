use std::collections;
use std::rand;
use std::rand::Rng;

// XXX: Make a loveletter namespace (figure out how to do this properly).

// XXX: Stop playing whack-a-mole with references & ownership and start
// actually *understanding* it.

#[deriving(PartialEq, PartialOrd, Eq, Ord, Show, Clone)]
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

#[deriving(Show)]
struct Game {
    // XXX: [rust]: Why can't I derive show while this has a size? Why can't I
    // make it a slice?
    //
    // XXX: I reckon I can make '_hands' an array without making the code much
    // more complicated. I wonder if that would matter.
    _hands: Vec<Option<Card>>,
    _stack: Vec<Card>,
}


impl Game {
    fn new() -> Game {
        Game::from_deck(Deck::new())
    }

    fn _player_set() -> collections::BitvSet {
        collections::BitvSet::from_bitv(collections::Bitv::with_capacity(4, true))
    }

    fn from_deck(deck: Deck) -> Game {
        let Deck(cards) = deck;
        let hand_end = NUM_PLAYERS + 1;
        Game {
            _hands: cards.slice(1, hand_end).iter().map(|&x| Some(x)).collect(),
            _stack: cards.slice_from(hand_end).iter().map(|&x| x).collect(),
        }
    }

    fn from_manual(hands: [Option<Card>, ..NUM_PLAYERS], deck: &[Card]) -> Result<Game, DeckError> {
        let stack: Vec<Card> = deck.iter().map(|&x| x).collect();
        let mut all_cards = stack.clone();
        for x in hands.as_slice().iter().filter_map(|&x| x) {
            all_cards.push(x);
        }
        let difference = subtract_vector(DECK.iter().map(|&x| x).collect(), all_cards);
        match difference {
            Some(_) => Ok(Game {
                _hands: hands.iter().map(|&x| x).collect(),
                _stack: stack,
            }),
            None    => Err(WrongCards),
        }
    }

    fn hands(&self) -> &[Option<Card>] {
        self._hands.as_slice()
    }

    fn deck(&self) -> &[Card] {
        self._stack.as_slice()
    }

    fn get_hand(&self, player: uint) -> Result<Card, PlayError> {
        // XXX: Maybe a good idea to return an error if the player is
        // protected by the priestess
        if player < self._hands.len() {
            match self._hands[player] {
                Some(card) => Ok(card),
                None => Err(InactivePlayer(player)),
            }
        } else {
            Err(InvalidPlayer(player))
        }
    }

    fn eliminate(&self, player: uint) -> Result<Game, PlayError> {
        match self.get_hand(player) {
            Err(e) => Err(e),
            Ok(..) => {
                let mut hands = self._hands.clone();
                hands.as_mut_slice()[player] = None;
                Ok(Game { _hands: hands, _stack: self._stack.clone() })
            }
        }
    }

    fn num_cards_remaining(&self) -> uint {
        self._stack.len()
    }

    fn active_players(&self) -> Vec<uint> {
        // XXX: I think we might only need this for testing, so fudge over it.
        let mut vec = Vec::with_capacity(NUM_PLAYERS);
        for (i, card) in self._hands.iter().enumerate() {
            match *card {
                Some(_) => vec.push(i),
                None    => ()
            }
        }
        vec
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

enum Action {
    UseGeneral(uint),
}

// XXX: Will probably make sense to move it into the Game object, but let's
// keep it separate for now.


// XXX: has to assume that the dealt_card is no longer on the stack, because
// Wizard will force another player to draw from the deck.

#[deriving(PartialEq, Eq, Show)]
enum PlayError {
    InvalidPlayer(uint),
    InvalidAction,
    InactivePlayer(uint),
}

// XXX: With Wizard, will need to check if they are forced to play the Princess.
fn judge(game: Game, dealt_card: Card, action: Action) -> Result<Game, PlayError> {
    // XXX: my spider sense is telling me this can be modeled as a
    // non-deterministic finite automata.
    match action {
        UseGeneral(target) => {
            match game.get_hand(target) {
                Err(e) => return Err(e),
                _      => (),
            }
            // XXX: need current player in order to swap. Assume it's 0 for now.
            let current_player = 0;
            let current_card = game.hands()[current_player];

            if !(current_card == Some(General) || dealt_card == General) {
                return Err(InvalidAction);
            }
            // XXX: might want to extract 'get the one that's not this' logic.
            let mut new_game = game;
            if current_card == Some(General) {
                new_game._hands.as_mut_slice()[current_player] = Some(dealt_card);
            }

            // XXX: need to update so priestess renders ineffective
            new_game._hands.as_mut_slice().swap(target, current_player);
            Ok(new_game)
        }
    }
}


#[test]
fn test_card_ordering() {
    assert!(Soldier <= Soldier);
    assert!(Soldier < Clown);
    assert!(Soldier < Princess);
}


#[test]
fn test_num_cards_remaining_on_new() {
    // make a new game, make sure that the number & kinds of cards matches the
    // rules (5 soldiers, 2 clowns, etc.)
    let g = Game::new();
    assert_eq!(11, g.num_cards_remaining())
}

#[test]
fn test_active_players_on_new() {
    let g = Game::new();
    assert_eq!(vec![0, 1, 2, 3], g.active_players());
}

#[test]
fn test_get_card_active_player() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    assert_eq!(g.get_hand(0), Ok(General));
}

#[test]
fn test_get_card_nonexistent_player() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    assert_eq!(g.get_hand(5), Err(InvalidPlayer(5)));
}

#[test]
fn test_get_card_inactive_player() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    assert_eq!(g.get_hand(2), Err(InactivePlayer(2)));
}

#[test]
fn test_no_hand_means_not_active() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    assert_eq!(vec![0, 1, 3], g.active_players());
}

#[test]
fn test_eliminate_player() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    let new_game = g.eliminate(0).unwrap();
    assert_eq!(vec![1, 3], new_game.active_players());
}

// XXX: [emacs] How do I jump to the last mark? glyph wrote about that once.

#[test]
fn test_eliminate_nonexistent_player() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    let error = g.eliminate(5).unwrap_err();
    assert_eq!(InvalidPlayer(5), error);
}

#[test]
fn test_eliminate_gone_player() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    let error = g.eliminate(2).unwrap_err();
    assert_eq!(InactivePlayer(2), error);
}


#[test]
fn test_all_cards_in_game() {
    // make a new game, make sure that the number & kinds of cards matches the
    // rules (5 soldiers, 2 clowns, etc.). One card is 'burned' before dealing.
    let g = Game::new();
    let mut full_deck: Vec<&Card> = DECK.iter().collect();
    full_deck.sort();
    let mut found_cards: Vec<Card> = g.deck().iter().map(|&x| x).collect();
    for card in g.hands().iter() {
        match *card {
            Some(c) => found_cards.push(c),
            None    => ()
        }
    }
    found_cards.sort();
    for card in found_cards.iter() {
        let found = full_deck.iter().position(|&c| c == card);
        match found {
            Some(i) => full_deck.swap_remove(i),
            None    => fail!("card in game that's not in deck: {}", card),
        };
    }
    assert_eq!(1, full_deck.len());
    found_cards.push(*full_deck[0]);
    assert!(is_valid_deck(found_cards.as_slice()));
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
    assert_eq!(
        cards.slice(1, 5)
            .iter()
            .map(|&x| Some(x))
            .collect::<Vec<Option<Card>>>()
            .as_slice(),
        g.hands());
    assert_eq!(cards.slice_from(5), g.deck());
}


#[test]
fn test_manual_game() {
    // XXX: How to specify eliminated players?

    // XXX: Will need to update to take current player, because it won't be
    // able to figure out when previous players were eliminated.
    let hands = [Some(Soldier), Some(Clown), Some(Soldier), Some(Princess)];
    let stack = [Soldier, Soldier, Minister];
    let game = Game::from_manual(hands, stack).unwrap();
    assert_eq!(hands.as_slice(), game.hands());
    assert_eq!(stack.as_slice(), game.deck().as_slice());
}

#[test]
fn test_invalid_manual_game() {
    let hands = [Some(Soldier), Some(Clown), Some(Soldier), Some(Princess)];
    let stack = [Soldier, Princess, Minister];
    let result = Game::from_manual(hands, stack);
    match result {
        Ok(_)  => fail!("Had two Princesses, should not be ok."),
        Err(e) => assert_eq!(e, WrongCards)
    }
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
fn test_general_swap() {
    let mut g = Game::from_manual(
        [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    assert_eq!(Wizard, next_card);
    assert_eq!(Some(General), g.hands()[0]);

    let next_game = judge(g, next_card, UseGeneral(3)).unwrap();
    assert_eq!(next_game.hands()[3], Some(Wizard));
    assert_eq!(next_game.hands()[0], Some(Priestess));
}

#[test]
fn test_general_swap_bad_target() {
    let mut g = Game::from_manual(
        [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let next_game = judge(g, next_card, UseGeneral(4));
    match next_game {
        Ok(_)  => fail!("Should not have succeeded"),
        Err(e) => assert_eq!(InvalidPlayer(4), e)
    }
}

#[test]
fn test_general_with_no_general() {
    let mut g = Game::from_manual(
        [Some(Soldier), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let next_game = judge(g, next_card, UseGeneral(2));
    match next_game {
        Ok(_)  => fail!("Should not have succeeded"),
        Err(e) => assert_eq!(InvalidAction, e)
    }
}

// XXX: [emacs] My work config has M-f and M-b break at underscores. It's
// extremely useful, and I should set it up in my home config.

#[test]
fn test_general_at_inactive_players() {
    let mut g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let next_game = judge(g, next_card, UseGeneral(2));
    match next_game {
        Ok(_)  => fail!("Should not have succeeded"),
        Err(e) => assert_eq!(InactivePlayer(2), e)
    }
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


// XXX: This is algorithmically slow and probably slow in other ways.
// I'm just doing the bare minimum, as I want to pop my yak-stack a little.
//
// NB: it's only order-preserving because it makes the tests easier.
fn subtract_vector<A: PartialEq>(xs: Vec<A>, ys: Vec<A>) -> Option<Vec<A>> {
    let mut zs = xs;
    for y in ys.iter() {
        let pos = zs.iter().position(|x| x == y);
        match pos {
            Some(i) => { zs.remove(i); }
            None => return None
        };
    }
    Some(zs)
}

#[test]
fn test_vector_diff_trivial() {
    let xs: Vec<int> = vec![];
    let ys = vec![];
    assert_eq!(Some(vec![]), subtract_vector(xs, ys))
}

#[test]
fn test_vector_diff_identity() {
    let xs: Vec<int> = vec![1, 2, 3];
    let ys = vec![];
    assert_eq!(Some(vec![1, 2, 3]), subtract_vector(xs, ys))
}

#[test]
fn test_vector_diff_removes() {
    let xs: Vec<int> = vec![1, 2, 3];
    let ys = vec![2];
    assert_eq!(Some(vec![1, 3]), subtract_vector(xs, ys))
}

#[test]
fn test_vector_diff_only_removes_one() {
    let xs: Vec<int> = vec![1, 2, 3, 2];
    let ys = vec![2];
    assert_eq!(Some(vec![1, 3, 2]), subtract_vector(xs, ys))
}

#[test]
fn test_vector_diff_contains_excess_elements() {
    let xs: Vec<int> = vec![1, 2, 3, 2];
    let ys = vec![2, 2, 2];
    assert_eq!(None, subtract_vector(xs, ys))
}

#[test]
fn test_vector_diff_contains_novel_elements() {
    let xs: Vec<int> = vec![1, 2, 3, 2];
    let ys = vec![4];
    assert_eq!(None, subtract_vector(xs, ys))
}
