use super::{Card, Direction, Book, Trade, Inventory, Order, Event, CL};

pub mod event_driven;
pub use event_driven::*;

pub mod generic;
pub use generic::GenericPlayer;

pub mod tilt;
pub use tilt::TiltInventory;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PlayerName {
    Spread,
    Seller,
    Taker,
    Noisy,
    WildestDreams,
    PickOff,
    TiltInventory,
    None,
}



