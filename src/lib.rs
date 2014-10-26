pub use deck::{Card, Soldier, Clown, Knight, Priestess, Wizard, General, Minister, Princess};

pub mod deck;
mod util;

// Game state:
// - discarded ('burnt') card
// - the remaining deck
// - each player's card
// - whether they are protected by priestess
// - each player's discard
//   - publicly available

// XXX: Should we wrap up 'Player'? Especially interesting if we have the Game
// store separate discarded card logs. Also useful for keeping 'Priestess'
// protection data.

// TODO: Data structure for all of the publicly visible actions in a game.
// Must be enough to reconstruct the whole game.


#[deriving(Show, PartialEq, Eq)]
pub struct Turn {
    pub player: uint,
    pub hand: deck::Card,
    pub draw: deck::Card,
}

impl Turn {
    pub fn new(player: uint, hand: deck::Card, draw: deck::Card) -> Turn {
        Turn {
            player: player,
            hand: hand,
            draw: draw,
        }
    }
}

#[deriving(Show, PartialEq, Eq, Clone)]
pub struct Game {
    _hands: Vec<Option<deck::Card>>,
    _stack: Vec<deck::Card>,
    _num_players: uint,
    // None before any hand is played. Not 100% sure I like this, because it
    // means we're always checking whether it's the first player's turn.
    // Alternative is to initialize the game with the first player having
    // drawn their card.
    _current_player: Option<uint>,
}


#[deriving(Show, PartialEq, Eq)]
pub enum GameError {
    InvalidPlayers(uint),
    BadDeck,
}


impl Game {
    pub fn new(num_players: uint) -> Option<Game> {
        Game::from_deck(num_players, deck::Deck::new())
    }

    fn num_players(&self) -> uint {
        self._num_players
    }

    fn current_player(&self) -> Option<uint> {
        self._current_player
    }

    fn valid_player_count(num_players: uint) -> bool {
        2 <= num_players && num_players <= 4
    }

    pub fn from_deck(num_players: uint, deck: deck::Deck) -> Option<Game> {
        if !Game::valid_player_count(num_players) {
            return None
        }
        let cards = deck.as_slice();
        let hand_end = num_players + 1;
        Some(Game {
            _hands: cards.slice(1, hand_end).iter().map(|&x| Some(x)).collect(),
            _stack: cards.slice_from(hand_end).iter().map(|&x| x).collect(),
            _num_players: num_players,
            _current_player: None,
        })
    }

    pub fn from_manual(hands: &[Option<deck::Card>], deck: &[deck::Card],
                       current_player: Option<uint>) -> Result<Game, GameError> {
        let num_players = hands.len();
        if !Game::valid_player_count(num_players) {
            return Err(InvalidPlayers(num_players));
        }
        let stack: Vec<deck::Card> = deck.iter().map(|&x| x).collect();
        let mut all_cards = stack.clone();
        for x in hands.as_slice().iter().filter_map(|&x| x) {
            all_cards.push(x);
        }
        if deck::is_valid_subdeck(all_cards.as_slice()) {
            Ok(Game {
                _hands: hands.iter().map(|&x| x).collect(),
                _stack: stack,
                _num_players: hands.len(),
                _current_player: current_player,
            })
        } else {
            Err(BadDeck)
        }
    }

    fn hands(&self) -> &[Option<deck::Card>] {
        self._hands.as_slice()
    }

    #[cfg(test)]
    fn deck(&self) -> &[deck::Card] {
        self._stack.as_slice()
    }

    fn get_hand(&self, player: uint) -> Result<deck::Card, PlayError> {
        // XXX: Maybe a good idea to return an error if the player is
        // protected by the priestess
        if player < self.num_players() {
            match self._hands[player] {
                Some(card) => Ok(card),
                None => Err(InactivePlayer(player)),
            }
        } else {
            Err(InvalidPlayer(player))
        }
    }

    fn eliminate(&self, player: uint) -> Result<Game, PlayError> {
        let mut new_game = self.clone();
        match self.get_hand(player) {
            Err(e) => { return Err(e); },
            Ok(..) => {
                let player = new_game._hands.get_mut(player);
                *player = None;
            }
        };
        Ok(new_game)
    }

    fn swap_hands(&self, p1: uint, p2: uint) -> Result<Game, PlayError> {
        let mut new_game = self.clone();
        match self.get_hand(p2).and(self.get_hand(p1)) {
            Err(e) => { return Err(e); },
            Ok(..) => {
                new_game._hands.as_mut_slice().swap(p1, p2);
            }
        };
        Ok(new_game)
    }

    fn discard_and_draw(&self, player: uint) -> Result<Game, PlayError> {
        // TODO: Check that they are not playing Princess. If they are,
        // eliminate them.
        let mut game = self.clone();
        let new_card = game._draw();
        match self.get_hand(player) {
            Err(e) => return Err(e),
            Ok(..) => {
                game._hands.as_mut_slice()[player] = new_card;
            }
        }
        Ok(game)
    }

    fn num_cards_remaining(&self) -> uint {
        self._stack.len()
    }

    fn _draw(&mut self) -> Option<deck::Card> {
        self._stack.pop()
    }

    fn draw(&self) -> (Game, Option<deck::Card>) {
        let mut new_game = self.clone();
        let card = new_game._draw();
        (new_game, card)
    }

    fn _next_player(&self) -> Option<uint> {
        let current = match self._current_player {
            None    => -1,
            Some(x) => x,
        };
        let num_players = self.num_players();
        range(1, num_players)
            .map(|i| (current + i) % num_players)
            .find(|i| self._hands[*i].is_some())
    }

    fn next_player(&self) -> (Game, Option<Turn>) {
        let mut new_game = self.clone();
        let card = new_game._draw();
        match card {
            None => (new_game, None),
            Some(c) => {
                match new_game._next_player() {
                    None => (new_game, None), // XXX: This branch not tested
                    Some(new_player) => {
                        new_game._current_player = Some(new_player);
                        let hand = new_game._hands[new_player];
                        (new_game, Some(Turn {
                            player: new_player,
                            draw: c,
                            hand: hand.expect("Activated disabled player"),
                        }))
                    }
                }
            }
        }
    }

    // TODO: Some routine for identifying the winners.


    fn eliminate_weaker(&self, p1: uint, p2: uint) -> Result<Action, PlayError> {
        match (self.get_hand(p1), self.get_hand(p2)) {
            (Ok(p1_card), Ok(p2_card)) =>
                match p1_card.cmp(&p2_card) {
                    Less    => Ok(EliminatePlayer(p1)),
                    Greater => Ok(EliminatePlayer(p2)),
                    Equal   => Ok(NoChange),
                },
            (Err(e), _) => Err(e),
            (_, Err(e)) => Err(e),
        }
    }

    fn apply_action(&self, action: Action) -> Result<Game, PlayError> {
        let new_game = self.clone();
        match action {
            NoChange => Ok(self.clone()),
            EliminatePlayer(i) => self.eliminate(i),
            SwapHands(p1, p2, _) => self.swap_hands(p1, p2),
            ForceDiscard(i) => self.discard_and_draw(i),
            ForceReveal(..) => Ok(new_game),
            EliminateWeaker(p1, p2) =>
                match self.eliminate_weaker(p1, p2) {
                    Ok(action) => self.apply_action(action),
                    Err(e) => Err(e),
                },
            EliminateOnGuess(p1, card) =>
                match self.get_hand(p1) {
                    Ok(deck::Soldier) => Err(BadGuess),
                    Ok(c) => self.apply_action(
                        if card == c {
                            EliminatePlayer(p1)
                        } else {
                            NoChange
                        }),
                    Err(err) => Err(err),
                }
        }
    }

    pub fn handle_turn(&self, f: |&Game, &Turn| -> (deck::Card, Play)) -> Result<Option<Game>, PlayError> {
        // TODO: UNTESTED:
        let (new_game, turn) = self.next_player();
        let mut new_game = new_game;
        let turn = match turn {
            None => return Ok(None),
            Some(turn) => turn,
        };
        // TODO: It is at this point which,
        // - priestess expires
        // - minister can bite you
        let (card, play) = f(&new_game, &turn);

        let action = match judge(&new_game, turn.player, turn.draw, (card, play)) {
            Ok(a) => a,
            Err(e) => return Err(e),
        };

        if card == turn.hand {
            let card = new_game._hands.get_mut(turn.player);
            *card = Some(turn.draw);
        }

        // XXX: Probably should return the action so that an external client can
        // infer what happened?
        new_game = match new_game.apply_action(action) {
            Ok(g) => g,
            Err(e) => return Err(e),
        };

        Ok(Some(new_game))
    }
}


fn minister_bust(a: deck::Card, b: deck::Card) -> bool {
    a == deck::Minister && b >= deck::Wizard || a >= deck::Wizard && b == deck::Minister
}



#[deriving(PartialEq, Eq, Show)]
pub enum Play {
    NoEffect,
    Attack(uint),
    Guess(uint, Card),
}

// XXX: I really want to have a data association from Cards to possible movies.


#[deriving(PartialEq, Eq, Show)]
pub enum PlayError {
    // Targeted a player who has never existed.
    InvalidPlayer(uint),
    // Tried to play a card that's not in the hand.
    CardNotFound(deck::Card, (deck::Card, deck::Card)),
    // Targeted a player who is no longer in the game.
    InactivePlayer(uint),
    // Tried to play a card against yourself.
    SelfTarget(uint, deck::Card),
    // Tried to play an action for a card that doesn't support it.
    BadActionForCard(Play, deck::Card),
    // Bad guess. You can't guess soldier.
    BadGuess,
}


/// The result of a play.
#[deriving(PartialEq, Eq, Show)]
pub enum Action {
    NoChange,
    // source, target, source card
    // source card is there in case we're swapping the card we just picked up.
    SwapHands(uint, uint, deck::Card),
    // You have lost
    EliminatePlayer(uint),
    // Discard your current card and draw a new one
    ForceDiscard(uint),
    // 2nd player shows their card to 1st.
    ForceReveal(uint, uint),
    EliminateWeaker(uint, uint),
    EliminateOnGuess(uint, deck::Card),
}


// XXX: Will probably make sense to move it into the Game object, but let's
// keep it separate for now.
fn judge(game: &Game, current_player: uint, dealt_card: deck::Card,
         play: (deck::Card, Play)) -> Result<Action, PlayError> {
    let current_card = match game.get_hand(current_player) {
        Ok(card) => card,
        Err(e) => return Err(e),
    };

    // Make sure we're targeting a valid, active player.
    match play {
        (_, Attack(target)) | (_, Guess(target, _))  => match game.get_hand(target) {
            Err(e) => return Err(e),
            _ => (),
        },
        _ => (),
    }

    let (played_card, play_data) = play;

    let unplayed_card = match util::other((current_card, dealt_card), played_card) {
        Some(card) => card,
        None => return Err(CardNotFound(played_card, (current_card, dealt_card))),
    };

    // Only need `unplayed_card` for General.
    play_to_action(current_player, played_card, unplayed_card, play_data)
}


/// Turn a play into an Action.
///
/// Translates a decision by a player to play a particular card in a
/// particular way into an Action that can be applied to the game.
///
/// Returns an error if that particular card, play combination is not valid.
fn play_to_action(
    current_player: uint, played_card: deck::Card, unplayed_card: deck::Card,
    play: Play) -> Result<Action, PlayError> {

    // XXX: Ideally, I'd express this with a data structure that mapped card,
    // play combinations to valid actions.

    match play {
        NoEffect => match played_card {
            deck::Priestess => Ok(NoChange),
            deck::Minister => Ok(NoChange),
            deck::Princess => Ok(EliminatePlayer(current_player)),
            _ => Err(BadActionForCard(play, played_card)),
        },
        Attack(target) => {
            if target == current_player {
                // TODO: Make sure you target yourself with a wizard. (Add tests).
                return Err(SelfTarget(target, played_card));
            }

            match played_card {
                deck::Clown => {
                    Ok(ForceReveal(current_player, target))
                },
                deck::Knight => {
                    Ok(EliminateWeaker(current_player, target))
                },
                deck::Wizard => {
                    Ok(ForceDiscard(target))
                },
                deck::General => {
                    Ok(SwapHands(current_player, target, unplayed_card))
                },
                _ => Err(BadActionForCard(play, played_card)),
            }
        }
        Guess(target, guessed_card) => {
            // TODO: make it so you cannot guess at yourself & add tests.
            // TODO: tests for guessing right, guessing wrong, targeting invalid person,
            // Guess action w/ non-soldier.
            match played_card {
                deck::Soldier => Ok(EliminateOnGuess(target, guessed_card)),
                _ => Err(BadActionForCard(play, played_card)),
            }
        }
    }
}


#[cfg(test)]
mod test {
    use deck::{Card, Soldier, Clown, Knight, Priestess, Wizard, General, Minister, Princess};
    use deck;

    use super::Game;
    use super::judge;
    use super::{NoChange, SwapHands, EliminatePlayer, ForceDiscard, ForceReveal, EliminateWeaker};
    use super::{Attack, Guess, NoEffect};
    use super::{InvalidPlayer, CardNotFound, InactivePlayer, SelfTarget, BadActionForCard};

    fn make_arbitrary_game() -> Game {
        Game::new(4).unwrap()
    }

    #[test]
    fn test_num_players() {
        assert_eq!(3, Game::new(3).unwrap().num_players());
    }

    #[test]
    fn test_too_few_players() {
        assert_eq!(None, Game::new(1));
    }

    #[test]
    fn test_too_many_players() {
        assert_eq!(None, Game::new(5));
    }

    #[test]
    fn test_current_player_at_start() {
        assert_eq!(None, make_arbitrary_game().current_player());
    }

    #[test]
    fn test_current_player_after_next() {
        let g = make_arbitrary_game();
        let (g2, _) = g.next_player();
        assert_eq!(Some(0), g2.current_player());
    }

    #[test]
    fn test_next_player_gets_draw() {
        let g = make_arbitrary_game();
        let (_, turn) = g.next_player();
        let super::Turn { player: p, draw: d, hand: _ } = turn.unwrap();
        let (_, expected) = g.draw();
        assert_eq!((p, d), (0, expected.unwrap()));
    }

    #[test]
    fn test_next_player_increments() {
        let g = Game::new(2).unwrap();
        let (g, _) = g.next_player();
        let (g, _) = g.next_player();
        assert_eq!(Some(1), g.current_player());
    }

    #[test]
    fn test_next_player_cycles() {
        let g = Game::new(2).unwrap();
        let (g, _) = g.next_player();
        let (g, _) = g.next_player();
        let (g, _) = g.next_player();
        assert_eq!(Some(0), g.current_player());
    }

    #[test]
    fn test_num_cards_remaining_on_new() {
        // make a new game, make sure that the number & kinds of cards matches the
        // rules (5 soldiers, 2 clowns, etc.)
        let g = make_arbitrary_game();
        assert_eq!(11, g.num_cards_remaining())
    }

    #[test]
    fn test_get_card_active_player() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
            [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
        assert_eq!(g.get_hand(0), Ok(General));
    }

    #[test]
    fn test_get_card_nonexistent_player() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
            [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
        assert_eq!(g.get_hand(5), Err(InvalidPlayer(5)));
    }

    #[test]
    fn test_get_card_inactive_player() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
        assert_eq!(g.get_hand(2), Err(InactivePlayer(2)));
    }

    #[test]
    fn test_eliminate_nonexistent_player() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
        let error = g.eliminate(5).unwrap_err();
        assert_eq!(InvalidPlayer(5), error);
    }

    #[test]
    fn test_eliminate_gone_player() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
        let error = g.eliminate(2).unwrap_err();
        assert_eq!(InactivePlayer(2), error);
    }

    #[test]
    fn test_skip_eliminated_player() {
        let g = Game::new(3).unwrap();
        let (g, _) = g.next_player();
        let g = g.eliminate(1).unwrap();
        let (g, t) = g.next_player();
        assert_eq!(g.current_player(), Some(2));
        assert_eq!(t.unwrap().player, 2);
    }


    #[test]
    fn test_swap_cards() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
        let new_game = g.swap_hands(0, 1).unwrap();
        assert_eq!(
            [Some(Clown), Some(General), None, Some(Priestess)].as_slice(),
            new_game.hands());
    }

    #[test]
    fn test_swap_cards_nonexistent() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier, Wizard], None).unwrap();
        let error = g.swap_hands(0, 5).unwrap_err();
        assert_eq!(InvalidPlayer(5), error);
        let error = g.swap_hands(5, 0).unwrap_err();
        assert_eq!(InvalidPlayer(5), error);
    }

    #[test]
    fn test_no_change() {
        let g = make_arbitrary_game();
        let new_g = g.apply_action(NoChange).unwrap();
        assert_eq!(g, new_g);
    }

    #[test]
    fn test_eliminate_action() {
        let g = Game::new(3).unwrap();
        let (g, _) = g.next_player();
        let new_g = g.apply_action(EliminatePlayer(1)).unwrap();
        let (_, t) = new_g.next_player();
        assert_eq!(2, t.unwrap().player);
    }

    #[test]
    fn test_force_swap() {
        let g = Game::from_manual(
            [Some(Soldier), Some(Clown), Some(Knight)],
            [Soldier, Minister, Princess, Soldier, General], None).unwrap();
        let (g, t) = g.next_player();
        let t = t.unwrap();
        let ours = t.hand;
        let theirs = g.get_hand(1).unwrap();
        let new_g = g.apply_action(SwapHands(0, 1, ours)).unwrap();
        assert_eq!(theirs, new_g.get_hand(0).unwrap());
        assert_eq!(ours, new_g.get_hand(1).unwrap());
    }

    #[test]
    fn test_all_cards_in_game() {
        // make a new game, make sure that the number & kinds of cards matches the
        // rules (5 soldiers, 2 clowns, etc.). One card is 'burned' before dealing.
        let g = make_arbitrary_game();
        let mut full_deck: Vec<Card> = deck::Deck::new().iter().map(|x| *x).collect();
        full_deck.sort();
        let mut found_cards: Vec<Card> = g.deck().iter().map(|x| *x).collect();
        for card in g.hands().iter() {
            match *card {
                Some(c) => found_cards.push(c),
                None    => ()
            }
        }
        found_cards.sort();
        for card in found_cards.iter() {
            let found = full_deck.iter().position(|c| c == card);
            match found {
                Some(i) => full_deck.swap_remove(i),
                None    => fail!("card in game that's not in deck: {}", card),
            };
        }
        assert_eq!(1, full_deck.len());
        found_cards.push(full_deck[0]);

        let mut fresh_deck: Vec<Card> = deck::Deck::new().iter().map(|x| *x).collect();
        fresh_deck.sort();
        found_cards.sort();
        assert_eq!(fresh_deck, found_cards);
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
        let deck = deck::Deck::from_slice(cards).unwrap();
        let num_players = 3u;
        let g = Game::from_deck(num_players, deck).unwrap();
        assert_eq!(
            cards.slice(1, num_players + 1)
                .iter()
                .map(|&x| Some(x))
                .collect::<Vec<Option<Card>>>()
                .as_slice(),
            g.hands());
        assert_eq!(cards.slice_from(num_players + 1), g.deck());
        assert_eq!(num_players, g.num_players());
    }

    #[test]
    fn test_manual_game() {
        let hands = [Some(Soldier), Some(Clown), Some(Soldier)];
        let stack = [Soldier, Soldier, Minister];
        let game = Game::from_manual(hands, stack, None).unwrap();
        assert_eq!(hands.as_slice(), game.hands());
        assert_eq!(stack.as_slice(), game.deck().as_slice());
        assert_eq!(hands.len(), game.num_players());
    }

    #[test]
    fn test_invalid_manual_game() {
        let hands = [Some(Soldier), Some(Clown), Some(Soldier), Some(Princess)];
        let stack = [Soldier, Princess, Minister];
        let result = Game::from_manual(hands, stack, None);
        match result {
            Ok(_)  => fail!("Had two Princesses, should not be ok."),
            Err(e) => assert_eq!(e, super::BadDeck)
        }
    }

    #[test]
    fn test_manual_game_bad_players() {
        assert_eq!(Err(super::InvalidPlayers(0)), Game::from_manual([], [], None));
    }


    #[test]
    fn test_minister_bust() {
        assert!(!super::minister_bust(Soldier, Soldier));
        assert!(super::minister_bust(Minister, Wizard));
        assert!(super::minister_bust(Minister, General));
        assert!(super::minister_bust(Minister, Princess));
        assert!(super::minister_bust(Wizard, Minister));
        assert!(super::minister_bust(General, Minister));
        assert!(super::minister_bust(Princess, Minister));
    }

    #[test]
    fn test_judge_invalid_player() {
        let g = make_arbitrary_game();
        let err = judge(&g, 5, Soldier, (Priestess, NoEffect)).unwrap_err();
        assert_eq!(InvalidPlayer(5), err);
    }

    #[test]
    fn test_judge_invalid_target_attack() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
            [Soldier, Minister, Princess, Soldier], None).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (General, Attack(4)));
        assert_eq!(InvalidPlayer(4), result.unwrap_err());
    }

    #[test]
    fn test_judge_invalid_target_guess() {
        let g = Game::from_manual(
            [Some(Soldier), Some(Clown), Some(Knight), Some(Priestess)],
            [Soldier, Minister, Princess, Soldier], None).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (Soldier, Guess(4, Minister)));
        assert_eq!(InvalidPlayer(4), result.unwrap_err());
    }

    #[test]
    fn test_judge_inactive_player_attack() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier], None).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (General, Attack(2)));
        assert_eq!(InactivePlayer(2), result.unwrap_err());
    }

    #[test]
    fn test_judge_inactive_player_guess() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier], None).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (General, Guess(2, Minister)));
        assert_eq!(InactivePlayer(2), result.unwrap_err());
    }

    #[test]
    fn test_judge_play_without_card() {
        let g = Game::from_manual(
            [Some(Soldier), Some(Clown), Some(Knight), Some(Priestess)],
            [Soldier, Minister, Princess, Soldier], None).unwrap();
        // Player 0 has a Wizard and a Soldier, but is trying to play a
        // General.
        let result = judge(&g, 0, Wizard, (General, Attack(2)));
        assert_eq!(
            CardNotFound(General, (Soldier, Wizard)), result.unwrap_err());
    }

    #[test]
    fn test_general_swap() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
            [Soldier, Minister, Princess, Soldier], None).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (General, Attack(3))).unwrap();
        assert_eq!(result, SwapHands(0, 3, Wizard));
    }

    #[test]
    fn test_self_targeting() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
            [Soldier, Minister, Princess, Soldier], None).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (General, Attack(0)));
        assert_eq!(SelfTarget(0, General), result.unwrap_err());
    }

    #[test]
    fn test_knight() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier], None).unwrap();
        let result = judge(&g, 0, Knight, (Knight, Attack(3)));
        assert_eq!(EliminateWeaker(0, 3), result.unwrap());
    }

    #[test]
    fn test_wizard() {
        let g = Game::from_manual(
            [Some(Wizard), Some(Soldier), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier, Knight], None).unwrap();
        let arbitrary_card = Soldier;
        let result = judge(&g, 0, arbitrary_card, (Wizard, Attack(1)));
        assert_eq!(ForceDiscard(1), result.unwrap());
    }

    #[test]
    fn test_clown() {
        let g = Game::from_manual(
            [Some(Clown), Some(Soldier), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier, Knight], None).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (Clown, Attack(1)));
        assert_eq!(ForceReveal(0, 1), result.unwrap());
    }

    #[test]
    fn test_non_attack() {
        let g = Game::from_manual(
            [Some(Clown), Some(Soldier), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier], None).unwrap();
        let arbitrary_card = Knight;
        let result = judge(&g, 1, arbitrary_card, (Soldier, Attack(0)));
        assert_eq!(BadActionForCard(Attack(0), Soldier), result.unwrap_err());
    }

    #[test]
    #[should_fail]
    fn test_weirdness() {
        // TODO: I encountered this behaviour when I was exploring the
        // interactive application. The bug is that player 0 eliminates
        // themselves with a bad knight play, and then play proceeds to player
        // 1, instead of correctly ending the game.
        //
        // I think the real solution is to start making integration tests that
        // exercise 'handle_turn'.
        let g = Game::from_manual(
            [Some(Priestess), Some(Minister)],
            [Wizard, Wizard, Soldier, Princess, Knight], Some(1)).unwrap();
        let (g, turn) = g.next_player();
        let mut g = g;
        let turn = turn.unwrap();
        let (card, play) = (Knight, Attack(1));
        let action = judge(&g, turn.player, turn.draw, (card, play)).unwrap();
        if card == turn.hand {
            let card = g._hands.get_mut(turn.player);
            *card = Some(turn.draw);
        }
        let new_game = g.apply_action(action).unwrap();
        assert_eq!(new_game.get_hand(0), Ok(Priestess));
    }

}
