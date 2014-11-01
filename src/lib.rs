pub use deck::{Card, Soldier, Clown, Knight, Priestess, Wizard, General, Minister, Princess};

use player::Player;

pub mod deck;
pub mod prompt;

mod player;
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
enum GameState {
    NotStarted,
    PlayerReady(uint),
}




#[deriving(Show, PartialEq, Eq, Clone)]
pub struct Game {
    _stack: Vec<deck::Card>,
    _players: Vec<Player>,
    // Not sure I like this. The current Game class only accepts a callback,
    // so a player is dealt a card and must respond in the same method. It is
    // always some player's turn unless it's the beginning or end. A different
    // concept would be to have a very high level game state enum which had
    // different kinds of values depending on whether the game was over or
    // not.
    //
    // e.g. Before the game, you have a Deck, a number of players and nothing
    // else. During the game, there are methods to draw a card, to play it,
    // and (probably) to inspect public state. After the game, the only thing
    // that can happen is you look at who the survivors are, what their cards
    // were, who the winner is, and what the burn card was.
    _current_player: GameState,
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

    pub fn num_players(&self) -> uint {
        self._players.len()
    }

    fn current_player(&self) -> Option<uint> {
        match self._current_player {
            NotStarted => None,
            PlayerReady(i) => Some(i)
        }
    }

    fn update_player(&self, player_id: uint, player: Player) -> Game {
        let mut new_game = self.clone();
        new_game._players[player_id] = player;
        new_game
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
        let hands: Vec<Option<deck::Card>> = cards.slice(1, hand_end).iter().map(|&x| Some(x)).collect();
        Some(Game {
            _stack: cards.slice_from(hand_end).iter().map(|&x| x).collect(),
            _current_player: NotStarted,
            _players: hands.iter().map(|&x| Player::new(x)).collect(),
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
            let state = match current_player {
                None => NotStarted,
                Some(i) => PlayerReady(i),
            };
            Ok(Game {
                _stack: stack,
                _current_player: state,
                _players: hands.iter().map(|&x| Player::new(x)).collect(),
            })
        } else {
            Err(BadDeck)
        }
    }

    #[cfg(test)]
    fn hands(&self) -> Vec<Option<deck::Card>> {
        self._players.iter().map(|&x| x.get_hand()).collect()
    }

    #[cfg(test)]
    fn deck(&self) -> &[deck::Card] {
        self._stack.as_slice()
    }

    fn get_player(&self, player: uint) -> Result<Player, PlayError> {
        if player < self.num_players() {
            let p = self._players[player];
            if p.active() {
                Ok(p)
            } else {
                Err(InactivePlayer(player))
            }
        } else {
            Err(InvalidPlayer(player))
        }
    }

    fn get_hand(&self, player: uint) -> Result<deck::Card, PlayError> {
        self.get_player(player).map(|p| p.get_hand().unwrap())
    }

    fn eliminate(&self, player: uint) -> Result<Game, PlayError> {
        match self.get_player(player) {
            Err(e) => { Err(e) },
            Ok(p) => {
                let (new_p, changed) = p.eliminate();
                if !changed {
                    Ok(self.clone())
                } else {
                    Ok(self.update_player(player, new_p))
                }
            }
        }
    }

    fn swap_hands(&self, p1: uint, p2: uint) -> Result<Game, PlayError> {
        match self.get_player(p2).and(self.get_player(p1)) {
            Err(e) => { Err(e) },
            Ok(..) => {
                let ((new_p1, new_p2), changed) = self._players[p1].swap_hands(self._players[p2]);
                if !changed {
                    Ok(self.clone())
                } else {
                    let mut new_game = self.clone();
                    new_game._players[p1] = new_p1;
                    new_game._players[p2] = new_p2;
                    Ok(new_game)
                }
            }
        }
    }

    fn protect(&self, p: uint) -> Result<Game, PlayError> {
        self.get_player(p).map(|player| self.update_player(p, player.protect(true)))
    }

    fn discard_and_draw(&self, player: uint) -> Result<Game, PlayError> {
        // TODO: Check that they are not playing Princess. If they are,
        // eliminate them.
        let mut game = self.clone();
        let new_card = game._draw();
        match self.get_hand(player) {
            Err(e) => return Err(e),
            Ok(..) => {
                game._players[player] = game._players[player].replace(new_card);
            }
        }
        Ok(game)
    }

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

    #[cfg(test)]
    fn num_cards_remaining(&self) -> uint {
        self._stack.len()
    }

    fn num_players_remaining(&self) -> uint {
        self._players.iter().filter(|p| p.active()).count()
    }

    fn _draw(&mut self) -> Option<deck::Card> {
        self._stack.pop()
    }

    fn _next_player(&self) -> Option<uint> {
        if self.num_players_remaining() <= 1 {
            None
        } else {
            let current_num = match self.current_player() {
                None => -1,
                Some(i) => i,
            };
            let num_players = self.num_players();
            range(1, num_players)
                .map(|i| (current_num + i) % num_players)
                .find(|i| self._players[*i].active())
        }
    }

    fn next_player(&self) -> (Game, Option<Turn>) {
        let mut new_game = self.clone();
        let card = new_game._draw();
        match card {
            None => (new_game, None),
            Some(c) => {
                let next_player = new_game._next_player();
                match next_player {
                    None => (self.clone(), None),
                    Some(new_player) => {
                        new_game._current_player = PlayerReady(new_player);
                        let hand = new_game._players[new_player].get_hand();
                        // Protection from the priestess expires when your
                        // turn begins.
                        new_game._players[new_player] = new_game._players[new_player].protect(false);
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

    /// At the end of the game, return players and their hands.
    fn survivors(&self) -> Vec<(uint, deck::Card)> {
        // TODO: Write tests
        // next_player essentially functions as a 'is game over' predicate.
        match self.next_player() {
            (_, Some(..)) => vec![],
            (_, None) => self._players
                .iter()
                .enumerate()
                .filter_map(
                    |(i, &p)| match p.get_hand() {
                        Some(y) => Some((i, y)),
                        None => None,
                    })
                .collect()
        }
    }

    pub fn winners(&self) -> Vec<(uint, deck::Card)> {
        let survivors = self.survivors();
        let mut ws = vec![];
        for x in util::maxima_by(&survivors, |&(_, card)| card).iter() {
            let &&(i, c) = x;
            ws.push((i, c))
        }
        ws
    }

    fn apply_action(&self, action: Action) -> Result<Game, PlayError> {
        match action {
            EliminateWeaker(i, j) | SwapHands(i, j, _) =>
                if self._players[i].protected() || self._players[j].protected() {
                    return Ok(self.clone());
                } else {
                    ()
                },
            EliminateOnGuess(i, _) | ForceDiscard(i) =>
                if self._players[i].protected() {
                    return Ok(self.clone());
                } else {
                    ()
                },
            _ => (),
        }
        match action {
            NoChange => Ok(self.clone()),
            Protect(i) => self.protect(i),
            EliminatePlayer(i) => self.eliminate(i),
            SwapHands(p1, p2, _) => self.swap_hands(p1, p2),
            ForceDiscard(i) => self.discard_and_draw(i),
            // XXX: The fact that this is indistinguishable from NoChange
            // means we've implemented it wrong.
            ForceReveal(..) => Ok(self.clone()),
            EliminateWeaker(p1, p2) =>
                match self.eliminate_weaker(p1, p2) {
                    Ok(action) => self.apply_action(action),
                    Err(e) => Err(e),
                },
            EliminateOnGuess(p1, card) =>
                // XXX: Factor some of this out into eliminate_on_guess method.
                if card == Soldier {
                    Err(BadGuess)
                } else {
                    match self.get_hand(p1) {
                        Ok(c) => self.apply_action(
                            if card == c {
                                EliminatePlayer(p1)
                            } else {
                                NoChange
                            }),
                        Err(err) => Err(err),
                    }
                },
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

        let action = if minister_bust(turn.draw, turn.hand) {
            EliminatePlayer(turn.player)
        } else {
            let (card, play) = f(&new_game, &turn);

            let action = match judge(&new_game, turn.player, turn.draw, (card, play)) {
                Ok(a) => a,
                Err(e) => return Err(e),
            };

            // Set the player's hand to the card they didn't play.
            if card == turn.hand {
                new_game._players[turn.player] = new_game._players[turn.player].replace(Some(turn.draw));
            }
            action
        };

        // XXX: Probably should return the action so that an external client can
        // infer what happened?
        match new_game.apply_action(action) {
            Ok(g) => Ok(Some(g)),
            Err(e) => return Err(e),
        }
    }
}


fn minister_bust(a: deck::Card, b: deck::Card) -> bool {
    match util::other((a, b), deck::Minister) {
        Some(deck::Wizard) | Some(deck::General) | Some(deck::Princess) => true,
        Some(deck::Minister) => panic!("Called with 2 ministers!"),
        _ => false,
    }
}


#[deriving(PartialEq, Eq, Show)]
pub enum Play {
    NoEffect,
    Attack(uint),
    Guess(uint, Card),
}


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
    Protect(uint),
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

    // Sort out which card we're playing, and which we're keeping.
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
            deck::Priestess => Ok(Protect(current_player)),
            deck::Minister => Ok(NoChange),
            deck::Princess => Ok(EliminatePlayer(current_player)),
            _ => Err(BadActionForCard(play, played_card)),
        },
        Attack(target) => {
            if target == current_player && played_card != deck::Wizard {
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
            if target == current_player {
                return Err(SelfTarget(target, played_card));
            }

            match played_card {
                deck::Soldier =>
                    if guessed_card == deck::Soldier {
                        Err(BadGuess)
                    } else {
                        Ok(EliminateOnGuess(target, guessed_card))
                    },
                _ => Err(BadActionForCard(play, played_card)),
            }
        }
    }
}


#[cfg(test)]
mod tests;
