use deck::Card;

#[deriving(PartialEq, Eq, Show)]
/// The play that accompanies a card.
pub enum Play {
    /// This card has no effect.
    NoEffect,
    /// Use this card to attack the specified player.
    Attack(uint),
    /// Use this card to guess that the specified player has a certain card.
    Guess(uint, Card),
}


#[deriving(PartialEq, Eq, Show)]
/// Represents an invalid action in a game, taken by a player.
pub enum PlayError {
    /// Targeted a player who has never existed.
    InvalidPlayer(uint),
    /// Tried to play a card that's not in the hand.
    CardNotFound(Card, (Card, Card)),
    /// Targeted a player who is no longer in the game.
    InactivePlayer(uint),
    /// Tried to play a card against yourself.
    SelfTarget(uint, Card),
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
    Protect(uint),
    /// source wants to swap hands with target
    SwapHands(uint, uint),
    /// You have lost
    EliminatePlayer(uint),
    /// Discard your current card and draw a new one
    ForceDiscard(uint),
    /// 2nd player shows their card to 1st.
    ForceReveal(uint, uint),
    /// Eliminate the player with the weaker hand.
    EliminateWeaker(uint, uint),
    /// Eliminate the player if they have the given card.
    EliminateOnGuess(uint, Card),
}


#[deriving(PartialEq, Eq, Show)]
pub enum Event {
    NoChange,
    Protected(uint),
    SwappedHands(uint, uint),
    PlayerEliminated(uint),
    ForcedDiscard(uint),
    ForcedReveal(uint, uint),
}


/// Turn a play into an Action.
///
/// Translates a decision by a player to play a particular card in a
/// particular way into an Action that can be applied to the game.
///
/// Returns an error if that particular `(card, play)` combination is not valid.
pub fn play_to_action(
    current_player: uint, played_card: Card, play: Play) -> Result<Action, PlayError> {

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
