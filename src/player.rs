use deck::Card;

#[deriving(Show, PartialEq, Eq, Clone)]
/// A player of Love Letter.
pub struct Player {
    _hand: Option<Card>,
    _protected: bool,
    _discard: Vec<Card>,
}


#[deriving(Show, PartialEq, Eq)]
pub enum Error {
    Inactive,
    BadGuess,
    NoSuchCard(Card, (Card, Card)),
}


impl Player {
    /// Create a new player with the given hand.
    pub fn new(hand: Option<Card>) -> Player {
        Player { _hand: hand, _protected: false, _discard: vec![] }
    }

    /// Is this player still playing?
    pub fn active(&self) -> bool {
        self._hand.is_some()
    }

    /// What has this player discarded?
    ///
    /// Last item is most-recently discarded.
    pub fn discards(&self) -> &[Card] {
        self._discard.as_slice()
    }

    /// Get the player's hand. Returns `None` if player is no longer playing.
    pub fn get_hand(&self) -> Option<Card> {
        self._hand
    }

    /// Set the protection status of the player.
    ///
    /// While they are 'protected', they are immune to all attacks.
    pub fn protect(&self, protected: bool) -> Result<Player, Error> {
        if self.active() {
            Ok(Player {
                _hand: self._hand,
                _protected: protected,
                _discard: self._discard.clone(),
            })
        } else {
            Err(Error::Inactive)
        }
    }

    /// Eliminate this player, making them no longer active and incapable of
    /// winning the game.
    ///
    /// Any card they had in their hand is immediately discarded.
    pub fn eliminate(&self) -> Result<Player, Error> {
        match self._hand {
            None => Err(Error::Inactive),
            Some(hand) =>
                Ok(if self._protected {
                    self.clone()
                } else {
                    self.replace(None).discard(hand)
                }),
        }
    }

    /// Eliminate this player if they've got the guessed card in their hand.
    pub fn eliminate_if_guessed(&self, guess: Card) -> Result<Player, Error> {
        if guess == Card::Soldier {
            return Err(Error::BadGuess)
        }
        match self._hand {
            None => Err(Error::Inactive),
            Some(card) =>
                if card == guess {
                    self.eliminate()
                } else {
                    Ok(self.clone())
                },
        }
    }

    /// Eliminate this player if their card is weaker than the other player's.
    /// Otherwise, eliminate the other player. If it's a draw, leave them both
    /// as they are.
    ///
    /// Returns a tuple of `(self, other)` where `self` and `other` are the
    /// updated versions. Only one will have changed.
    pub fn eliminate_if_weaker(&self, other: &Player) -> Result<(Player, Player), Error> {
        match (self._hand, other._hand) {
            (Some(my_card), Some(their_card)) => {
                if self._protected {
                    Ok((self.clone(), other.clone()))
                } else {
                    Ok(match my_card.cmp(&their_card) {
                        Less => (self.replace(None), other.clone()),
                        Greater => (self.clone(), other.replace(None)),
                        Equal => (self.clone(), other.clone())
                    })
                }
            }
            _ => Err(Error::Inactive),
        }
    }

    /// Swap hands with the other player.
    pub fn swap_hands(&self, other: &Player) -> Result<(Player, Player), Error> {
        if !self.active() {
            Err(Error::Inactive)
        } else if self._protected {
            Ok((self.clone(), other.clone()))
        } else {
            Ok((self.replace(other._hand), other.replace(self._hand)))
        }
    }

    /// Given that we were dealt `dealt` and chose to play `chosen`, discard
    /// `chosen` and put whatever card we didn't play in our hand.
    pub fn play_card(&self, dealt: Card, chosen: Card) -> Result<Player, Error> {
        match self._hand {
            None => Err(Error::Inactive),
            Some(hand) =>
                if chosen == hand {
                    Ok(self.discard(chosen).replace(Some(dealt)))
                } else if chosen == dealt {
                    Ok(self.discard(chosen))
                } else {
                    Err(Error::NoSuchCard(chosen, (hand, dealt)))
                },
        }
    }

    /// Discard current card and replace it with `new_card`.
    ///
    /// Different from `play_card` in that there might not be a new card
    /// available for us, in which case we're out of the game. Also, we never
    /// have to do this if we're protected.
    pub fn discard_and_draw(&self, new_card: Option<Card>) -> Result<Player, Error> {
        match self._hand {
            None => Err(Error::Inactive),
            Some(hand) =>
                if self._protected {
                    Ok(self.clone())
                } else {
                    match hand {
                        // XXX: Also need to make sure that when we eliminate in this
                        // case, that we return the new_card to the stack.
                        Card::Princess => self.eliminate(),
                        _ => Ok(self.discard(hand).replace(new_card)),
                    }
                },
        }
    }

    fn discard(&self, card: Card) -> Player {
        let mut new_discard = self._discard.clone();
        new_discard.push(card);
        Player {
            _hand: self._hand,
            _protected: self._protected,
            _discard: new_discard,
        }
    }

    fn replace(&self, card: Option<Card>) -> Player {
        Player {
            _hand: card,
            _protected: self._protected,
            _discard: self._discard.clone(),
        }
    }
}


#[cfg(test)]
mod test {
    use deck;
    use deck::Card;
    use super::{Player, Error};

    #[test]
    fn test_eliminate_gone_player() {
        let p = Player::new(None);
        let error = p.eliminate().unwrap_err();
        assert_eq!(Error::Inactive, error);
    }

    #[test]
    fn test_discards_empty() {
        let p = Player::new(Some(Card::Wizard));
        let expected: &[Card] = &[];
        assert_eq!(expected, p.discards());
    }

    #[test]
    fn test_play_hand_updates_hand() {
        let p = Player::new(Some(Card::Wizard));
        let new_p = p.play_card(Card::Priestess, Card::Wizard).unwrap();
        assert_eq!(Some(Card::Priestess), new_p.get_hand());
    }

    #[test]
    fn test_play_hand_discards() {
        let p = Player::new(Some(Card::Wizard));
        let new_p = p.play_card(Card::Priestess, Card::Wizard).unwrap();
        let expected: &[Card] = &[Card::Wizard];
        assert_eq!(expected, new_p.discards());
    }

    #[test]
    fn test_play_dealt_leaves_hand() {
        let p = Player::new(Some(Card::Wizard));
        let new_p = p.play_card(Card::Priestess, Card::Priestess).unwrap();
        assert_eq!(Some(Card::Wizard), new_p.get_hand());
    }

    #[test]
    fn test_play_dealt_discards() {
        let p = Player::new(Some(Card::Wizard));
        let new_p = p.play_card(Card::Priestess, Card::Priestess).unwrap();
        let expected: &[Card] = &[Card::Priestess];
        assert_eq!(expected, new_p.discards());
    }

    #[test]
    fn test_force_discard_updates_discard() {
        let p = Player::new(Some(Card::Knight));
        let new_p = p.discard_and_draw(Some(Card::Clown)).unwrap();
        let expected: &[Card] = &[Card::Knight];
        assert_eq!(expected, new_p.discards());
    }

    #[test]
    fn test_force_discard_princess_updates_discard() {
        let p = Player::new(Some(Card::Princess));
        let new_p = p.discard_and_draw(Some(Card::Clown)).unwrap();
        let expected: &[Card] = &[Card::Princess];
        assert_eq!(expected, new_p.discards());
    }

    #[test]
    fn test_eliminate_discards() {
        let p = Player::new(Some(Card::Princess));
        let new_p = p.eliminate().unwrap();
        let expected: &[Card] = &[Card::Princess];
        assert_eq!(expected, new_p.discards());
    }

}
