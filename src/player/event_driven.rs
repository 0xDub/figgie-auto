use super::{Card, Direction, Book, Trade, Inventory, Order, Event, CL, PlayerName};
use kanal::AsyncSender;
use tokio::sync::broadcast::Sender;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Instant;

pub struct EventDrivenPlayer {
    pub name: PlayerName,
    pub timer: Instant,
    pub verbose: bool,
    pub inventory: Inventory,
    pub trades: Vec<Trade>,
    pub event_receiver: Sender<Event>,
    pub order_sender: Arc<AsyncSender<Order>>,
    pub trading: Arc<AtomicBool>,
}

impl EventDrivenPlayer {
    pub fn new(
        player_name: PlayerName,
        verbose: bool,
        event_receiver: Sender<Event>,
        order_sender: Arc<AsyncSender<Order>>,
    ) -> Self {
        Self {
            name: player_name,
            timer: Instant::now(),
            verbose,
            inventory: Inventory::new(),
            trades: Vec::new(),
            event_receiver,
            order_sender,
            trading: Arc::new(AtomicBool::new(false)),
        }
    }



    pub async fn start(&mut self) {
        let mut event_receiver = self.event_receiver.subscribe();

        loop {
            if let Ok(event) = event_receiver.recv().await {
                match event {
                    Event::Update(update) => {

                        let trading_flag = self.trading.load(Ordering::Acquire);
                        if !trading_flag {
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            continue;
                        }

                        if let Some(trade) = update.trade { 
                            self.trades.push(trade.clone()); // push trade for historical reasons (if we want to analyze) & update inventory
                            if trade.buyer == self.name {
                                self.inventory.change(trade.card, 1);
                            } else if trade.seller == self.name {
                                self.inventory.change(trade.card, -1);
                            }
                        }

                        let seconds_left = 240 - self.timer.elapsed().as_secs();

                        let inventory = self.inventory;

                        let spades_book = update.spades;
                        let clubs_book = update.clubs;
                        let diamonds_book = update.diamonds;
                        let hearts_book = update.hearts;

                        // be careful with EventDriven, this can lead to a snowball of events if the # of orders leads from 1 -> many
                        // core logic goes here


                        
                        
                        match self.name {
                            PlayerName::PickOff => {
                                self.pick_off(seconds_left, inventory.spades, spades_book, Card::Spade).await;
                                self.pick_off(seconds_left, inventory.clubs, clubs_book, Card::Club).await;
                                self.pick_off(seconds_left, inventory.diamonds, diamonds_book, Card::Diamond).await;
                                self.pick_off(seconds_left, inventory.hearts, hearts_book, Card::Heart).await;
                            },
                            _ => {}
                        }

                    }
                    Event::DealCards(players_inventory) => {
                        self.inventory = players_inventory.get(&self.name).unwrap().clone();
                        
                        if self.verbose {
                            println!("{}[+] {:?} |:| Received cards: {:?}{}", CL::DullGreen.get(), self.name, self.inventory, CL::End.get());
                        }
                        
                        self.trading.store(true, Ordering::Release);
                        self.timer = Instant::now();
                    },
                    Event::EndRound => {
                        self.trading.store(false, Ordering::Release);
                    }
                }
            }
        }
    }



    pub async fn send_order(&self, price: usize, direction: Direction, card: &Card) {
        let order = Order {
            player_name: self.name.clone(),
            price,
            direction,
            card: card.clone(),
        };

        if self.verbose {
            println!("{:?} |:| Sending order: {:?}", self.name, order);
        }

        if let Err(e) = self.order_sender.send(order).await {
            println!("[!] {:?} |:| Error sending order: {:?}", self.name, e);
        }
    }

    pub fn get_max_price_from_seconds(&self, seconds_left: u64) -> usize {
        if seconds_left < 20 {
            0
        } else if seconds_left < 40 {
            1
        } else if seconds_left < 60 {
            2
        } else if seconds_left < 120 {
            3
        } else {
            4
        }
    }

    pub async fn pick_off(&self, seconds_left: u64, inventory: usize, book: Book, card: Card) {
        if inventory <= 2 {
            if book.ask.price < self.get_max_price_from_seconds(seconds_left) && book.ask.player_name != self.name {
                self.send_order(book.ask.price, Direction::Buy, &card).await;
            }
        }

        if inventory > 0 {
            if book.ask.price > 5 && book.ask.player_name != self.name {
                self.send_order(book.ask.price - 1, Direction::Sell, &card).await;
            }
        }
    }

}