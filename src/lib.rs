use std::collections;
use std::rand;
use std::rand::Rng;

mod util;

// XXX: [rust] Make a loveletter namespace (figure out how to do this properly).

// XXX: [rust] Stop playing whack-a-mole with references & ownership and start
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


struct Deck(Vec<Card>);

#[deriving(Show, PartialEq, Eq)]
enum DeckError {
    WrongCards,
    WrongNumber(uint),
}

impl Deck {
    /// Returns a new, shuffled deck.
    fn new() -> Deck {
        // Safe to unwrap because we know that DECK has CARDS_IN_DECK
        // elements.
        Deck::from_slice(&DECK).unwrap().shuffled()
    }

    fn from_slice(cards: &[Card]) -> Result<Deck, DeckError> {
        if cards.len() != CARDS_IN_DECK {
            return Err(WrongNumber(cards.len()));
        } else if is_valid_deck(cards) {
            Ok(Deck(cards.iter().map(|x| *x).collect()))
        } else {
            Err(WrongCards)
        }
    }

    fn shuffled(&self) -> Deck {
        let &Deck(ref cards) = self;
        let mut new_cards = cards.clone();
        let mut rng = rand::task_rng();
        rng.shuffle(new_cards.as_mut_slice());
        Deck(new_cards)
    }
}

fn is_valid_deck(deck: &[Card]) -> bool {
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
        let difference = util::subtract_vector(DECK.iter().map(|&x| x).collect(), all_cards);
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

    fn swap_hands(&self, p1: uint, p2: uint) -> Result<Game, PlayError> {
        match self.get_hand(p2).and(self.get_hand(p1)) {
            Err(e) => Err(e),
            Ok(..) => {
                let mut hands = self._hands.clone();
                hands.as_mut_slice().swap(p1, p2);
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

#[deriving(PartialEq, Eq, Show)]
enum Play {
    Attack(uint),
}

// XXX: I really want to have a data association from Cards to possible movies.


#[deriving(PartialEq, Eq, Show)]
enum PlayError {
    // Targeted a player who has never existed.
    InvalidPlayer(uint),
    // Tried to play a card that's not in the hand.
    CardNotFound(Card, (Card, Card)),
    // Targeted a player who is no longer in the game.
    InactivePlayer(uint),
    // Tried to play a card against yourself.
    SelfTarget(uint, Card),
    // Tried to play an action for a card that doesn't support it.
    BadActionForCard(Play, Card),
}


/// The result of a play.
#[deriving(PartialEq, Eq, Show)]
enum Action {
    NoChange,
    // source, target, source card
    // source card is there in case we're swapping the card we just picked up.
    SwapHands(uint, uint, Card),
    // You have lost
    EliminatePlayer(uint),
    // Discard your current card and draw a new one
    ForceDiscard(uint),
    // Show your card (from, to, value)
    ForceReveal(uint, uint, Card)
}

// XXX: Probably would have been a good idea to write down the notation for a
// game before I started all of this.

// XXX: With Wizard, will need to check if they are forced to play the Princess.

// XXX: Will probably make sense to move it into the Game object, but let's
// keep it separate for now.
fn judge(game: Game, current_player: uint, dealt_card: Card,
         play: (Card, Play)) -> Result<Action, PlayError> {
    // XXX: my spider sense is telling me this can be modeled as a
    // non-deterministic finite automata.
    let current_card = match game.get_hand(current_player) {
        Ok(card) => card,
        Err(e) => return Err(e),
    };

    let (played_card, play_data) = play;

    let unplayed_card = match util::other((current_card, dealt_card), played_card) {
        Some(card) => card,
        None       => return Err(
            CardNotFound(played_card, (current_card, dealt_card))),
    };

    match play_data {
        Attack(target) => {
            if target == current_player {
                return Err(SelfTarget(target, played_card));
            }
            let target_card = match game.get_hand(target) {
                Err(e)   => return Err(e),
                Ok(card) => card,
            };

            match played_card {
                Clown => {
                    Ok(ForceReveal(current_player, target, target_card))
                },
                Knight => {
                    match unplayed_card.cmp(&target_card) {
                        Less    => Ok(EliminatePlayer(current_player)),
                        Greater => Ok(EliminatePlayer(target)),
                        Equal   => Ok(NoChange),
                    }
                },
                Wizard => {
                    Ok(ForceDiscard(target))
                },
                General => {
                    // XXX: maybe need to take priestess into account here
                    Ok(SwapHands(current_player, target, unplayed_card))
                },
                _ => Err(BadActionForCard(play_data, played_card)),
            }
        }
    }
}


// XXX: [rust]: It's getting unwieldy having all of these tests in the same
// text file. I wonder if there's a good way to break them up, ala Python.

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
fn test_swap_cards() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    let new_game = g.swap_hands(0, 1).unwrap();
    assert_eq!(
        [Some(Clown), Some(General), None, Some(Priestess)].as_slice(),
        new_game.hands());
}

#[test]
fn test_swap_cards_nonexistent() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    let error = g.swap_hands(0, 5).unwrap_err();
    assert_eq!(InvalidPlayer(5), error);
    let error = g.swap_hands(5, 0).unwrap_err();
    assert_eq!(InvalidPlayer(5), error);
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
    let deck = Deck::from_slice(cards).unwrap();
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
fn test_judge_invalid_player() {
    let g = Game::from_manual(
        [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    let err = judge(g, 5, Soldier, (General, Attack(2))).unwrap_err();
    assert_eq!(InvalidPlayer(5), err);
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

    let result = judge(g, 0, next_card, (General, Attack(3))).unwrap();
    assert_eq!(result, SwapHands(0, 3, Wizard));
}

#[test]
fn test_general_swap_bad_target() {
    let mut g = Game::from_manual(
        [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 0, next_card, (General, Attack(4)));
    assert_eq!(InvalidPlayer(4), result.unwrap_err());
}

#[test]
fn test_general_with_no_general() {
    let mut g = Game::from_manual(
        [Some(Soldier), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 0, next_card, (General, Attack(2)));
    assert_eq!(
        CardNotFound(General, (Soldier, Wizard)), result.unwrap_err());
}

#[test]
fn test_self_targeting() {
    let mut g = Game::from_manual(
        [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 0, next_card, (General, Attack(0)));
    assert_eq!(SelfTarget(0, General), result.unwrap_err());
}

#[test]
fn test_general_at_inactive_players() {
    let mut g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 0, next_card, (General, Attack(2)));
    assert_eq!(InactivePlayer(2), result.unwrap_err());
}

#[test]
fn test_knight_win() {
    let mut g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Knight]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 0, next_card, (Knight, Attack(3)));
    assert_eq!(EliminatePlayer(3), result.unwrap());
}

#[test]
fn test_knight_lose() {
    let mut g = Game::from_manual(
        [Some(General), Some(Clown), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Knight]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 1, next_card, (Knight, Attack(3)));
    assert_eq!(EliminatePlayer(1), result.unwrap());
}

#[test]
fn test_knight_draw() {
    let mut g = Game::from_manual(
        [Some(Soldier), Some(Soldier), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Knight]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 0, next_card, (Knight, Attack(1)));
    assert_eq!(NoChange, result.unwrap());
}

#[test]
fn test_knight_no_card() {
    let mut g = Game::from_manual(
        [Some(Soldier), Some(Soldier), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 0, next_card, (Knight, Attack(1)));
    assert_eq!(CardNotFound(Knight, (Soldier, Wizard)), result.unwrap_err());
}

#[test]
fn test_knight_invalid_player() {
    let mut g = Game::from_manual(
        [Some(Soldier), Some(Soldier), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Knight]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 0, next_card, (Knight, Attack(5)));
    assert_eq!(InvalidPlayer(5), result.unwrap_err());
}

#[test]
fn test_knight_inactive_player() {
    let mut g = Game::from_manual(
        [Some(Soldier), Some(Soldier), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Knight]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 0, next_card, (Knight, Attack(2)));
    assert_eq!(InactivePlayer(2), result.unwrap_err());
}

#[test]
fn test_wizard() {
    let mut g = Game::from_manual(
        [Some(Wizard), Some(Soldier), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Knight]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 0, next_card, (Wizard, Attack(1)));
    assert_eq!(ForceDiscard(1), result.unwrap());
}

#[test]
fn test_clown() {
    let mut g = Game::from_manual(
        [Some(Clown), Some(Soldier), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Knight]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 0, next_card, (Clown, Attack(1)));
    assert_eq!(ForceReveal(0, 1, Soldier), result.unwrap());
}

#[test]
fn test_non_attack() {
    let mut g = Game::from_manual(
        [Some(Clown), Some(Soldier), None, Some(Priestess)],
        [Soldier, Minister, Princess, Soldier, Knight]).unwrap();
    // XXX: Messing with internals: a sign of bad design!
    let next_card = g._stack.pop().unwrap();
    let result = judge(g, 1, next_card, (Soldier, Attack(0)));
    assert_eq!(BadActionForCard(Attack(0), Soldier), result.unwrap_err());
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


