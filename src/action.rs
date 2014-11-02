use deck;

#[deriving(PartialEq, Eq, Show)]
/// The play that accompanies a card.
pub enum Play {
    /// This card has no effect.
    NoEffect,
    /// Use this card to attack the specified player.
    Attack(uint),
    /// Use this card to guess that the specified player has a certain card.
    Guess(uint, deck::Card),
}


#[deriving(PartialEq, Eq, Show)]
/// Represents an invalid action in a game, taken by a player.
pub enum PlayError {
    /// Targeted a player who has never existed.
    InvalidPlayer(uint),
    /// Tried to play a card that's not in the hand.
    CardNotFound(deck::Card, (deck::Card, deck::Card)),
    /// Targeted a player who is no longer in the game.
    InactivePlayer(uint),
    /// Tried to play a card against yourself.
    SelfTarget(uint, deck::Card),
    /// Tried to play an action for a card that doesn't support it.
    BadActionForCard(Play, deck::Card),
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
    EliminateOnGuess(uint, deck::Card),
}


/// Turn a play into an Action.
///
/// Translates a decision by a player to play a particular card in a
/// particular way into an Action that can be applied to the game.
///
/// Returns an error if that particular `(card, play)` combination is not valid.
pub fn play_to_action(
    current_player: uint, played_card: deck::Card, play: Play) -> Result<Action, PlayError> {

    // XXX: Ideally, I'd express this with a data structure that mapped card,
    // play combinations to valid actions.

    match play {
        NoEffect => match played_card {
            deck::Priestess => Ok(Protect(current_player)),
            deck::Minister => Ok(NoChange),
            // XXX: Another way to do this is to return NoChange here and have
            // `Player` be responsible for eliminating self on Princess discard.
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
                    Ok(SwapHands(current_player, target))
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
