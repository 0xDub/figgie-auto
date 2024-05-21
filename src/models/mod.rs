use super::PlayerName;

pub mod event;
pub use event::*;
pub mod book;
pub use book::*;
pub mod inventory;
pub use inventory::*;
pub mod order;
pub use order::*;


#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Card {
    Spade,
    Club,
    Diamond,
    Heart,
}

impl Card {
    pub fn get_other_cards(&self) -> (Card, Card, Card) { // common_suite, suit_1, suit_2
        match self {
            Card::Spade => (Card::Club, Card::Diamond, Card::Heart),
            Card::Club => (Card::Spade, Card::Diamond, Card::Heart),
            Card::Heart => (Card::Diamond, Card::Spade, Card::Club),
            Card::Diamond => (Card::Heart, Card::Spade, Card::Club)
        }
    }

    pub fn get_goal_suit(&self) -> Card {
        match self {
            Card::Spade => Card::Club,
            Card::Club => Card::Spade,
            Card::Heart => Card::Diamond,
            Card::Diamond => Card::Heart
        }
    }
}



