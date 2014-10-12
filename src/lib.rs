mod deck;
mod util;

// Game state:
// - discarded ('burnt') card
// - the remaining deck
// - each player's card
// - whether they are protected by priestess
// - each player's discard
//   - publicly available

// XXX: Should we wrap up 'Player'?

#[deriving(Show, PartialEq, Eq, Clone)]
pub struct Game {
    // XXX: [rust]: Why can't I derive show while this has a size? Why can't I
    // make it a slice?
    //
    // XXX: I reckon I can make '_hands' an array without making the code much
    // more complicated. I wonder if that would matter.
    _hands: Vec<Option<deck::Card>>,
    _stack: Vec<deck::Card>,
    _num_players: uint,
    // None before any hand is played. Not 100% sure I like this, because it
    // means we're always checking whether it's the first player's turn.
    // Alternative is to initialize the game with the first player having
    // drawn their card.
    _current_player: Option<uint>,
}


#[deriving(Show, PartialEq, Eq, PartialOrd, Ord)]
enum GameError {
    InvalidPlayers(uint),
    BadDeck,
}


impl Game {
    pub fn new(num_players: uint) -> Option<Game> {
        Game::from_deck(num_players, deck::Deck::new())
    }

    pub fn num_players(&self) -> uint {
        self._num_players
    }

    pub fn current_player(&self) -> Option<uint> {
        self._current_player
    }

    pub fn current_hand(&self) -> Option<deck::Card> {
        self._current_player.and_then(|i| self.get_hand(i).ok())
    }

    fn valid_player_count(num_players: uint) -> bool {
        2 <= num_players && num_players <= 4
    }

    fn from_deck(num_players: uint, deck: deck::Deck) -> Option<Game> {
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

    #[cfg(test)]
    fn from_manual(hands: &[Option<deck::Card>], deck: &[deck::Card]) -> Result<Game, GameError> {
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
                _current_player: None,
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

    pub fn get_hand(&self, player: uint) -> Result<deck::Card, PlayError> {
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

    fn num_cards_remaining(&self) -> uint {
        self._stack.len()
    }

    pub fn _draw(&mut self) -> Option<deck::Card> {
        self._stack.pop()
    }

    fn draw(&self) -> (Game, Option<deck::Card>) {
        let mut new_game = self.clone();
        let card = new_game._draw();
        (new_game, card)
    }

    pub fn next_player(&self) -> (Game, Option<(uint, deck::Card)>) {
        let mut new_game = self.clone();
        let card = new_game._draw();
        let new_player = match new_game._current_player {
            Some(n) => (n + 1) % new_game._hands.len(),
            None => 0
        };
        new_game._current_player = Some(new_player);
        (new_game, card.map(|c| (new_player, c)))
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


fn minister_bust(a: deck::Card, b: deck::Card) -> bool {
    a == deck::Minister && b >= deck::Wizard || a >= deck::Wizard && b == deck::Minister
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
pub enum Play {
    Attack(uint),
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
    // Show your card (from, to, value)
    ForceReveal(uint, uint, deck::Card)
}

// XXX: Probably would have been a good idea to write down the notation for a
// game before I started all of this.

// XXX: With Wizard, will need to check if they are forced to play the Princess.

// XXX: Will probably make sense to move it into the Game object, but let's
// keep it separate for now.
pub fn judge(game: &Game, current_player: uint, dealt_card: deck::Card,
             play: (deck::Card, Play)) -> Result<Action, PlayError> {
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
                // XXX: You can target yourself with a wizard.
                return Err(SelfTarget(target, played_card));
            }
            let target_card = match game.get_hand(target) {
                Err(e)   => return Err(e),
                Ok(card) => card,
            };

            match played_card {
                deck::Clown => {
                    Ok(ForceReveal(current_player, target, target_card))
                },
                deck::Knight => {
                    match unplayed_card.cmp(&target_card) {
                        Less    => Ok(EliminatePlayer(current_player)),
                        Greater => Ok(EliminatePlayer(target)),
                        Equal   => Ok(NoChange),
                    }
                },
                deck::Wizard => {
                    Ok(ForceDiscard(target))
                },
                deck::General => {
                    // XXX: maybe need to take priestess into account here
                    Ok(SwapHands(current_player, target, unplayed_card))
                },
                _ => Err(BadActionForCard(play_data, played_card)),
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
    use super::{NoChange, SwapHands, EliminatePlayer, ForceDiscard, ForceReveal};
    use super::{Attack};
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
        let (_, draw) = g.next_player();
        let (_, expected) = g.draw();
        assert_eq!(draw, Some((0, expected.unwrap())));
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
        // XXX: Will need to update to take current player, because it won't be
        // able to figure out when previous players were eliminated.
        let hands = [Some(Soldier), Some(Clown), Some(Soldier)];
        let stack = [Soldier, Soldier, Minister];
        let game = Game::from_manual(hands, stack).unwrap();
        assert_eq!(hands.as_slice(), game.hands());
        assert_eq!(stack.as_slice(), game.deck().as_slice());
        assert_eq!(hands.len(), game.num_players());
    }

    #[test]
    fn test_invalid_manual_game() {
        let hands = [Some(Soldier), Some(Clown), Some(Soldier), Some(Princess)];
        let stack = [Soldier, Princess, Minister];
        let result = Game::from_manual(hands, stack);
        match result {
            Ok(_)  => fail!("Had two Princesses, should not be ok."),
            Err(e) => assert_eq!(e, super::BadDeck)
        }
    }

    #[test]
    fn test_manual_game_bad_players() {
        assert_eq!(Err(super::InvalidPlayers(0)), Game::from_manual([], []));
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
        let g = Game::from_manual(
            [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
            [Soldier, Minister, Princess, Soldier, Wizard]).unwrap();
        let err = judge(&g, 5, Soldier, (General, Attack(2))).unwrap_err();
        assert_eq!(InvalidPlayer(5), err);
    }


    #[test]
    fn test_general_swap() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
            [Soldier, Minister, Princess, Soldier]).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (General, Attack(3))).unwrap();
        assert_eq!(result, SwapHands(0, 3, Wizard));
    }

    #[test]
    fn test_general_swap_bad_target() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
            [Soldier, Minister, Princess, Soldier]).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (General, Attack(4)));
        assert_eq!(InvalidPlayer(4), result.unwrap_err());
    }

    #[test]
    fn test_general_with_no_general() {
        let g = Game::from_manual(
            [Some(Soldier), Some(Clown), Some(Knight), Some(Priestess)],
            [Soldier, Minister, Princess, Soldier]).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (General, Attack(2)));
        assert_eq!(
            CardNotFound(General, (Soldier, Wizard)), result.unwrap_err());
    }

    #[test]
    fn test_self_targeting() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), Some(Knight), Some(Priestess)],
            [Soldier, Minister, Princess, Soldier]).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (General, Attack(0)));
        assert_eq!(SelfTarget(0, General), result.unwrap_err());
    }

    #[test]
    fn test_general_at_inactive_players() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier]).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (General, Attack(2)));
        assert_eq!(InactivePlayer(2), result.unwrap_err());
    }

    #[test]
    fn test_knight_win() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier]).unwrap();
        let result = judge(&g, 0, Knight, (Knight, Attack(3)));
        assert_eq!(EliminatePlayer(3), result.unwrap());
    }

    #[test]
    fn test_knight_lose() {
        let g = Game::from_manual(
            [Some(General), Some(Clown), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier]).unwrap();
        let result = judge(&g, 1, Knight, (Knight, Attack(3)));
        assert_eq!(EliminatePlayer(1), result.unwrap());
    }

    #[test]
    fn test_knight_draw() {
        let g = Game::from_manual(
            [Some(Soldier), Some(Soldier), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier]).unwrap();
        let result = judge(&g, 0, Knight, (Knight, Attack(1)));
        assert_eq!(NoChange, result.unwrap());
    }

    #[test]
    fn test_knight_no_card() {
        let g = Game::from_manual(
            [Some(Soldier), Some(Soldier), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier]).unwrap();
        let result = judge(&g, 0, Wizard, (Knight, Attack(1)));
        assert_eq!(CardNotFound(Knight, (Soldier, Wizard)), result.unwrap_err());
    }

    #[test]
    fn test_knight_invalid_player() {
        let g = Game::from_manual(
            [Some(Soldier), Some(Soldier), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier]).unwrap();
        let result = judge(&g, 0, Knight, (Knight, Attack(5)));
        assert_eq!(InvalidPlayer(5), result.unwrap_err());
    }

    #[test]
    fn test_knight_inactive_player() {
        let g = Game::from_manual(
            [Some(Soldier), Some(Soldier), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier]).unwrap();
        let result = judge(&g, 0, Knight, (Knight, Attack(2)));
        assert_eq!(InactivePlayer(2), result.unwrap_err());
    }

    #[test]
    fn test_wizard() {
        let g = Game::from_manual(
            [Some(Wizard), Some(Soldier), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier, Knight]).unwrap();
        let arbitrary_card = Soldier;
        let result = judge(&g, 0, arbitrary_card, (Wizard, Attack(1)));
        assert_eq!(ForceDiscard(1), result.unwrap());
    }

    #[test]
    fn test_clown() {
        let g = Game::from_manual(
            [Some(Clown), Some(Soldier), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier, Knight]).unwrap();
        let arbitrary_card = Wizard;
        let result = judge(&g, 0, arbitrary_card, (Clown, Attack(1)));
        assert_eq!(ForceReveal(0, 1, Soldier), result.unwrap());
    }

    #[test]
    fn test_non_attack() {
        let g = Game::from_manual(
            [Some(Clown), Some(Soldier), None, Some(Priestess)],
            [Soldier, Minister, Princess, Soldier]).unwrap();
        let arbitrary_card = Knight;
        let result = judge(&g, 1, arbitrary_card, (Soldier, Attack(0)));
        assert_eq!(BadActionForCard(Attack(0), Soldier), result.unwrap_err());
    }

    #[test]
    fn test_draw_card() {
        let g = Game::from_manual(
            [Some(Clown), Some(Soldier)],
            [Soldier, Minister, Princess, Soldier, Knight]).unwrap();
        let (new_game, next_card) = g.draw();
        let expected: &[Card] = [Soldier, Minister, Princess, Soldier];
        assert_eq!(Some(Knight), next_card);
        assert_eq!(expected, new_game.deck());
    }

    #[test]
    fn test_draw_card_no_more_cards() {
        let g = Game::from_manual([Some(Clown), Some(Soldier)], []).unwrap();
        let (new_game, next_card) = g.draw();
        assert_eq!(None, next_card);
        assert_eq!(g, new_game);
    }

}
