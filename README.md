
# love letter

Just trying to implement the rules.

## notes on rules

Four players. All cards used. One card burnt. Each player dealt a single card.

5 x Soldier: name a player & a non-soldier card, named player loses if correct (1)

2 x Clown: See opponent's hand (2)

2 x Knight: Compare hands with opponent. Lowest card loses. (3)

2 x Priestess: Immune to attack (4)

2 x Wizard: Opponent discards hand (5)

1 x General: Swap hand (6)

1 x Minister: Lose if hand > 12 (7)

1 x Princess: Lose if played or forced to reveal (8)

Game ends when all players are eliminated, or when there are no more cards to
draw and the final player has played.

Winner is either the last player standing, or the player with the
highest-valued card.

## notes on implementation

Essentially focused on implementing minimal logic for adjudicating games.

Phase Two is implementing client layer that does card counting and has a slot
for a user-provided callback.

### representing actions

Two options.

1. Have a `Play` and associate each play with a card.

```
    enum Play {
        Guess(Card, Player),
        Attack(Player),
        Defend,
    }

```

Don't think there's a data-driven way to do this in rust.

2. Have an `Action` type that duplicates cards.

```
    enum Action {
        // name a player & a non-soldier card, named player loses if correct
        UseSoldier(Card, Player),
        // See opponent's hand
        UseClown(Player),
        UseKnight(Player),
        UsePriestess,
        UseWizard(Player),
        UseGeneral(Player),
        UseMinister,
        UsePrincess,
    }
```

## Notation

A game *starts* with a shuffled deck that looks like this:

    [Soldier, Soldier, Soldier, Soldier, Soldier,
     Clown, Clown, Knight, Knight, Priestess, Priestess,
     Wizard, Wizard, General, Minister, Princess]

The first card is burned, the remaining top 5 cards are dealt, each to one
player.

Player 1 draws a card (a Clown), and then must choose to play either a Soldier
or a Clown.

## Events

TODO: Incorporate these into the program.


* player $N plays $CARD with $ACTION
* Soldier: player $N does not have $CARD (inferrable)
* Soldier: player $N has $CARD (inferrable)
* Clown: player $N reveals card to player $M
* Clown: player $N has $CARD (only revealed to one person)
* Knight: player $N & $M secretly compared, and player $M lost with $CARD
* Knight: player $N & $M secretly compare, $N has $CARD, $M has $CARD2
* Wizard: player $N discards $CARD and draws another
* General: player $N swaps cards with player $M (inferrable)
* Minister: player $N busted with Minister and $CARD

## Let's try again, shall we?

There's a shuffled deck of cards.

There's a group of players.

Each player gets a card.

When it's their turn, they draw a card.

If the card is a Minister and they have a Wizard, General, or Princess, their
out.

Otherwise, they choose a card (one of two) to play. They place it face down in
front of them and choose what to do with it.

If it's a Minister, Princess, or Priestess, they do nothing.

If it's a General, Wizard, Knight, or Clown, they choose a person to attack
with it.

If it's a Soldier, they choose a person to attack and make a guess as to what
they have.

### Princess

They lose. Players see they've lost, and that they played a princess.

Princess, NoEffect, EliminatePlayer(i)

### Minister

Nothing ever happens.

If drawn, might BustOut.

### General

If protected, NoChange
Otherwise SwapHands

### Wizard

If protected, NoChange
Otherwise ForceDiscard

If ForceDiscard causes Princess to be discarded, targeted player is eliminated

### Priestess

Protected

### Knight

If protected, NoChange
Compare hands:
 if equal, NoChange
 otherwise, lesser Eliminated

### Clown

If protected, NoChange
Otherwise, player can see card
Actual game not changed

### Soldier

If protected, NoChange
Guess correct: EliminatePlayer
Incorrect: NoChange

## Result of a turn

Either:

a) Busting out before playing
b) Player, Card, Play, and ...
NoChange (also communicate if was because of Protected? Can be inferred)
Protect
EliminatePlayer
SwapHands
ForceDiscard
ForceReveal

ForceDiscard can trigger another 'event', when the one forced discards the
Princess.

All players learn:
- who played
- the card they played
- their intended action
- the result of their action

The person who played learns:
- On ForceReveal, the value of the card in a player's hand

### TODO

* make a function that applies this union type to game
  * still incorporated into `handle_turn`, maybe OK?
* update `handle_turn` to use this
