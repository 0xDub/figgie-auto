use super::{Card, PlayerName};


#[derive(Debug, Clone)]
pub enum Direction {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub player_name: PlayerName,
    pub price: usize,
    pub direction: Direction,
    pub card: Card,
}