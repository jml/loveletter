use deck::Card;
use player_id::PlayerId;


#[deriving(PartialEq, Eq, Show, Copy)]
/// The play that accompanies a card.
pub enum Play {
    /// This card has no effect.
    NoEffect,
    /// Use this card to attack the specified player.
    Attack(PlayerId),
    /// Use this card to guess that the specified player has a certain card.
    Guess(PlayerId, Card),
}


#[deriving(PartialEq, Eq, Show)]
/// Represents an invalid action in a game, taken by a player.
pub enum PlayError {
    /// Targeted a player who has never existed.
    InvalidPlayer(PlayerId),
    /// Tried to play a card that's not in the hand.
    CardNotFound(Card, (Card, Card)),
    /// Targeted a player who is no longer in the game.
    InactivePlayer(PlayerId),
    /// Tried to play a card against yourself.
    SelfTarget(PlayerId, Card),
    /// Tried to play an action for a card that doesn't support it.
    BadActionForCard(Play, Card),
    /// Bad guess. You can't guess soldier.
    BadGuess,
}


/// The result of a play.
#[deriving(PartialEq, Eq, Show)]
pub enum Action {
    /// Nothing happens.
    NoChange,
    /// Mark player as protected.
    Protect(PlayerId),
    /// source wants to swap hands with target
    SwapHands(PlayerId, PlayerId),
    /// You have lost
    EliminatePlayer(PlayerId),
    /// Discard your current card and draw a new one
    ForceDiscard(PlayerId),
    /// 2nd player shows their card to 1st.
    ForceReveal(PlayerId, PlayerId),
    /// Eliminate the player with the weaker hand.
    EliminateWeaker(PlayerId, PlayerId),
    /// Eliminate the player if they have the given card.
    EliminateOnGuess(PlayerId, Card),
}


#[deriving(PartialEq, Eq, Show, Copy)]
pub enum Event {
    NoChange,
    Protected(PlayerId),
    SwappedHands(PlayerId, PlayerId),
    PlayerEliminated(PlayerId),
    ForcedDiscard(PlayerId),
    ForcedReveal(PlayerId, PlayerId),
}


/// Turn a play into an Action.
///
/// Translates a decision by a player to play a particular card in a
/// particular way into an Action that can be applied to the game.
///
/// Returns an error if that particular `(card, play)` combination is not valid.
pub fn play_to_action(
    current_player: PlayerId, played_card: Card, play: Play) -> Result<Action, PlayError> {

    // XXX: Ideally, I'd express this with a data structure that mapped card,
    // play combinations to valid actions.

    // XXX: Do I even *need* Action, now that I've got Event. It provides nice
    // separation between cards and what the cards do, and allows me to do
    // some validation, but does that justify the complexity?

    match play {
        Play::NoEffect => match played_card {
            Card::Priestess => Ok(Action::Protect(current_player)),
            Card::Minister => Ok(Action::NoChange),
            // Another way to do this is to return NoChange here and have
            // `Player` be responsible for eliminating self on Princess
            // discard.
            Card::Princess => Ok(Action::EliminatePlayer(current_player)),
            _ => Err(PlayError::BadActionForCard(play, played_card)),
        },
        Play::Attack(target) => {
            if target == current_player && played_card != Card::Wizard {
                return Err(PlayError::SelfTarget(target, played_card));
            }

            match played_card {
                Card::Clown => {
                    Ok(Action::ForceReveal(current_player, target))
                },
                Card::Knight => {
                    Ok(Action::EliminateWeaker(current_player, target))
                },
                Card::Wizard => {
                    Ok(Action::ForceDiscard(target))
                },
                Card::General => {
                    Ok(Action::SwapHands(current_player, target))
                },
                _ => Err(PlayError::BadActionForCard(play, played_card)),
            }
        }
        Play::Guess(target, guessed_card) => {
            if target == current_player {
                return Err(PlayError::SelfTarget(target, played_card));
            }

            match played_card {
                Card::Soldier =>
                    if guessed_card == Card::Soldier {
                        Err(PlayError::BadGuess)
                    } else {
                        Ok(Action::EliminateOnGuess(target, guessed_card))
                    },
                _ => Err(PlayError::BadActionForCard(play, played_card)),
            }
        }
    }
}
