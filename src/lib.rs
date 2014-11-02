/// loveletter: implementation of [Love Letter](http://boardgamegeek.com/boardgame/129622/love-letter)

pub use deck::{Card, Soldier, Clown, Knight, Priestess, Wizard, General, Minister, Princess};
pub use action::{Play, PlayError, NoEffect, Attack, Guess, InvalidPlayer, InactivePlayer, BadGuess};

use player::Player;

pub mod deck;
pub mod prompt;

mod action;
mod player;
mod util;

// TODO: Data structure for all of the publicly visible actions in a game.
// Must be enough to reconstruct the whole game.


#[deriving(Show, PartialEq, Eq)]
pub struct Turn {
    pub player: uint,
    pub hand: deck::Card,
    pub draw: deck::Card,
}


#[deriving(Show, PartialEq, Eq, Clone)]
/// Possible states of a round of Love Letter.
///
/// ### Notes
///
/// XXX: Not sure I (jml) like this. The current Game class only accepts a
/// callback, so a player is dealt a card and must respond in the same method.
/// It is always some player's turn unless it's the beginning or end. A
/// different concept would be to have a very high level game state enum which
/// had different kinds of values depending on whether the game was over or
/// not.
///
/// e.g. Before the game, you have a Deck, a number of players and nothing
/// else. During the game, there are methods to draw a card, to play it,
/// and (probably) to inspect public state. After the game, the only thing
/// that can happen is you look at who the survivors are, what their cards
/// were, who the winner is, and what the burn card was.
enum GameState {
    NotStarted,
    PlayerReady(uint, deck::Card),
}



#[deriving(Show, PartialEq, Eq, Clone)]
/// Represents a single round of Love Letter.
pub struct Game {
    /// The remaining cards in the deck.
    _stack: Vec<deck::Card>,
    /// All of the players of the game. The size does not change once the game is constructed.
    _players: Vec<Player>,
    /// The current state of the game.
    _current: GameState,
}


#[deriving(Show, PartialEq, Eq)]
/// Errors that can occur while constructing a Game.
pub enum GameError {
    /// Specified an invalid number of players.
    InvalidPlayers(uint),
    /// The given cards do not form a valid deck.
    BadDeck,
}


impl Game {
    // TODO: Provide some way of getting the burnt card when play is over.

    // TODO: Create a state validator, that guarantees that no cards have been
    // created or destroyed.

    // TODO: Create a nice formatter that shows what's visible to a particular
    // player.

    /// Create a new game with a randomly shuffled deck.
    ///
    /// Will return None if given an invalid number of players.
    pub fn new(num_players: uint) -> Option<Game> {
        Game::from_deck(num_players, deck::Deck::new())
    }

    /// Create a new game given an already-shuffled deck.
    ///
    /// Will return None if given an invalid number of players.
    pub fn from_deck(num_players: uint, deck: deck::Deck) -> Option<Game> {
        if !Game::valid_player_count(num_players) {
            return None
        }
        let cards = deck.as_slice();
        let hand_end = num_players + 1;
        let hands: Vec<Option<deck::Card>> = cards.slice(1, hand_end).iter().map(|&x| Some(x)).collect();
        Some(Game {
            _stack: cards.slice_from(hand_end).iter().map(|&x| x).collect(),
            _current: NotStarted,
            _players: hands.iter().map(|&x| Player::new(x)).collect(),
        })
    }

    /// Create a new, in-progress game.
    ///
    /// `hands` is a slice of player hands. If the card is None, then that
    /// player is assumed to have been eliminated already. Otherwise, that's
    /// what's in their hand.
    ///
    /// `deck` is a stack of remaining cards in the deck. When players draw
    /// cards, they'll draw from the end.
    ///
    /// If `current_player` is `None`, then assume the game hasn't started.
    /// Otherwise (and this is a bit broken), the next player to play is the
    /// one **after** the one given here. e.g. `Some(0)` means it's player 1's
    /// turn next.
    pub fn from_manual(hands: &[Option<deck::Card>], deck: &[deck::Card],
                       current_player: Option<uint>) -> Result<Game, GameError> {
        let num_players = hands.len();
        if !Game::valid_player_count(num_players) {
            return Err(InvalidPlayers(num_players));
        }
        let mut stack: Vec<deck::Card> = deck.iter().map(|&x| x).collect();
        let mut all_cards = stack.clone();
        for x in hands.as_slice().iter().filter_map(|&x| x) {
            all_cards.push(x);
        }
        if deck::is_valid_subdeck(all_cards.as_slice()) {
            let state = match current_player {
                None => NotStarted,
                Some(i) => {
                    match stack.pop() {
                        Some(card) => PlayerReady(i, card),
                        None => { return Err(BadDeck); }
                    }
                }
            };
            Ok(Game {
                _stack: stack,
                _current: state,
                _players: hands.iter().map(|&x| Player::new(x)).collect(),
            })
        } else {
            Err(BadDeck)
        }
    }

    fn valid_player_count(num_players: uint) -> bool {
        2 <= num_players && num_players <= 4
    }

    /// Number of players in this game.
    pub fn num_players(&self) -> uint {
        self._players.len()
    }

    #[cfg(test)]
    fn num_cards_remaining(&self) -> uint {
        self._stack.len()
    }

    /// Number of active players still playing.
    fn num_players_remaining(&self) -> uint {
        self._players.iter().filter(|p| p.active()).count()
    }

    fn current_player(&self) -> Option<uint> {
        match self._current {
            NotStarted => None,
            PlayerReady(i, _) => Some(i)
        }
    }

    /// At the end of the game, return players and their hands.
    fn survivors(&self) -> Vec<(uint, deck::Card)> {
        // next_player essentially functions as a 'is game over' predicate.
        match self.next_player() {
            (_, Some(..)) => vec![],
            (_, None) => self._players
                .iter()
                .enumerate()
                .filter_map(
                    |(i, ref p)| match p.get_hand() {
                        Some(y) => Some((i, y)),
                        None => None,
                    })
                .collect()
        }
    }

    /// At the end of the game, return all winners and their hands.
    pub fn winners(&self) -> Vec<(uint, deck::Card)> {
        let survivors = self.survivors();
        let mut ws = vec![];
        for x in util::maxima_by(&survivors, |&(_, card)| card).iter() {
            let &&(i, c) = x;
            ws.push((i, c))
        }
        ws
    }

    fn get_player(&self, player_id: uint) -> Result<&Player, action::PlayError> {
        if player_id < self.num_players() {
            let ref p = self._players[player_id];
            if p.active() {
                Ok(p)
            } else {
                Err(action::InactivePlayer(player_id))
            }
        } else {
            Err(action::InvalidPlayer(player_id))
        }
    }

    fn get_hand(&self, player_id: uint) -> Result<deck::Card, action::PlayError> {
        self.get_player(player_id).map(|p| p.get_hand().unwrap())
    }

    fn update_player(&self, player_id: uint, player: Player) -> Game {
        let mut new_game = self.clone();
        new_game._players[player_id] = player;
        new_game
    }

    fn update_player_by(&self, player_id: uint, updater: |&Player| -> Result<Player, player::Error>) -> Result<Game, action::PlayError> {
        self.get_player(player_id)
            .map(updater)
            .and_then(
                |result| match result {
                    Ok(new_player) => Ok(self.update_player(player_id, new_player)),
                    Err(player::Inactive) => Err(action::InactivePlayer(player_id)),
                    Err(player::BadGuess) => Err(action::BadGuess),
                    Err(player::NoSuchCard(c, d)) => Err(action::CardNotFound(c, d)),
                })
    }

    fn update_two_players_by(&self, p1_id: uint, p2_id: uint, updater: |&Player, &Player| -> Result<(Player, Player), player::Error>) -> Result<Game, action::PlayError> {
        match (self.get_player(p1_id), self.get_player(p2_id)) {
            (Ok(player1), Ok(player2)) => {
                match updater(player1, player2) {
                    Ok((new_player1, new_player2)) => {
                        let mut new_game = self.clone();
                        new_game._players[p1_id] = new_player1;
                        new_game._players[p2_id] = new_player2;
                        Ok(new_game)
                    },
                    Err(player::Inactive) => Err(action::InactivePlayer(p2_id)),
                    Err(e) => panic!(e),
                }
            },
            (_, Err(e)) | (Err(e), _) => Err(e),
        }
    }

    #[cfg(test)]
    fn hands(&self) -> Vec<Option<deck::Card>> {
        self._players.iter().map(|ref x| x.get_hand()).collect()
    }

    #[cfg(test)]
    fn deck(&self) -> &[deck::Card] {
        self._stack.as_slice()
    }

    fn draw(&self) -> (Game, Option<deck::Card>) {
        let mut g = self.clone();
        let c = g._stack.pop();
        (g, c)
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
        match (self._next_player(), self.draw()) {
            (Some(new_player_id), (game, Some(c))) => {
                let mut new_game = game;
                new_game._current = PlayerReady(new_player_id, c);
                // Protection from the priestess expires when your
                // turn begins.
                new_game = new_game
                    .update_player_by(new_player_id, |p| p.protect(false))
                    .ok().expect("Activated disabled player");
                let hand = new_game.get_hand(new_player_id).ok().expect("Activated disabled player");
                (new_game, Some(Turn {
                    player: new_player_id,
                    draw: c,
                    hand: hand,
                }))
            },
            _ => (self.clone(), None),
        }
    }

    fn apply_action(&self, action: action::Action) -> Result<Game, action::PlayError> {
        match action {
            action::NoChange => Ok(self.clone()),
            action::Protect(i) => self.update_player_by(i, |player| player.protect(true)),
            action::EliminatePlayer(i) => self.update_player_by(i, |p| p.eliminate()),
            action::SwapHands(src, tgt) => self.update_two_players_by(
                tgt, src, |tgt_player, src_player| tgt_player.swap_hands(src_player)),
            action::ForceDiscard(i) => {
                let (game, new_card) = self.draw();
                game.update_player_by(i, |p| p.discard_and_draw(new_card))
            },
            // XXX: The fact that this is indistinguishable from NoChange
            // means we've implemented it wrong.
            action::ForceReveal(..) => Ok(self.clone()),
            action::EliminateWeaker(src, tgt) => self.update_two_players_by(
                tgt, src, |tgt_player, src_player| tgt_player.eliminate_if_weaker(src_player)),
            action::EliminateOnGuess(p1, card) =>
                self.update_player_by(p1, |p| p.eliminate_if_guessed(card))
        }
    }

    /// Crank the handle of a loveletter game.
    ///
    /// Takes a function which is given the game, the current player, the card
    /// they were dealt and the card in their hand. That function must return
    /// a card to play and the `Play` associated with it.
    ///
    /// `handle_turn` makes sure everything is valid and returns the new `Game`.
    ///
    /// If the game is now over, will return `Ok(None)`. If not, will return
    /// `Ok(Some(new_game))`.
    pub fn handle_turn(&self, f: |&Game, &Turn| -> (deck::Card, action::Play)) -> Result<Option<Game>, action::PlayError> {
        let (new_game, turn) = self.next_player();
        let turn = match turn {
            None => return Ok(None),
            Some(turn) => turn,
        };

        let (new_game, action) = if minister_bust(turn.draw, turn.hand) {
            (new_game, action::EliminatePlayer(turn.player))
        } else {
            // Find out what they'd like to play.
            let (card, play) = f(&new_game, &turn);

            // Update their hand and the played card.
            let new_game = match new_game.update_player_by(turn.player, |p| p.play_card(turn.draw, card)) {
                Err(e) => { return Err(e); }
                Ok(player) => { player }
            };

            let action = match action::play_to_action(turn.player, card, play) {
                Ok(a) => a,
                Err(e) => return Err(e),
            };

            (new_game, action)
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




#[cfg(test)]
mod tests;
