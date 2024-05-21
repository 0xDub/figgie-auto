use super::{Card, Book, Inventory, PlayerName};
use std::collections::HashMap;


#[derive(Debug, Clone)]
pub struct Trade {
    pub card: Card,
    pub price: usize,
    pub buyer: PlayerName,
    pub seller: PlayerName,
}


#[derive(Debug, Clone)]
pub struct Update {
    pub spades: Book,
    pub clubs: Book,
    pub diamonds: Book,
    pub hearts: Book,
    pub trade: Option<Trade>,
}

#[derive(Debug, Clone)]
pub enum Event {
    Update(Update),
    DealCards(HashMap<PlayerName, Inventory>),
    EndRound,
}