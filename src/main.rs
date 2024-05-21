use std::sync::Arc;

pub mod utils;
pub use utils::*;

pub mod models;
pub use models::*;

pub mod match_maker;
pub use match_maker::MatchMaker;

pub mod player;
pub use player::PlayerName;
pub use player::generic::GenericPlayer;
pub use player::even_driven::EventDrivenPlayer;

use crate::player::TiltInventory;


fn main() {

    const STARTING_BALANCE: usize = 500;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build runtime");
    runtime.block_on(async {


        println!("|==============================================|");
        println!("|{}          Welcome to Figgie Auto!             {}|", CL::Teal.get(), CL::End.get());
        println!("|        ---------------------------           |");
        println!("|    This is an algorithmic twist on Jane      |");
        println!("|   Street's 'Figgie'. However, instead of     |");
        println!("|  playing manually, setup some participants   |");
        println!("|  and see how they interact with each other.  |");
        println!("|       See 'player' for dev framework         |");
        println!("|                                              |");
        println!("|{}   -  All credit goes to Jane Street  -       {}|", CL::DullTeal.get(), CL::End.get());
        println!("|==============================================|\n");

        println!("The games are beginning\n");


        let mut handles = Vec::new();

        let (tx, rx) = kanal::unbounded_async::<Order>();
        let match_maker_order_receiver = Arc::new(rx);
        let player_1_order_sender = Arc::new(tx);
        let player_2_order_sender = Arc::clone(&player_1_order_sender);
        let player_3_order_sender = Arc::clone(&player_1_order_sender);
        let player_4_order_sender = Arc::clone(&player_1_order_sender);
        let player_5_order_sender = Arc::clone(&player_1_order_sender);


        let (match_maker_event_sender, _) = tokio::sync::broadcast::channel::<Event>(100);
        let player_1_event_receiver = match_maker_event_sender.clone();
        let player_2_event_receiver = match_maker_event_sender.clone();
        let player_3_event_receiver = match_maker_event_sender.clone();
        let player_4_event_receiver = match_maker_event_sender.clone();
        let player_5_event_receiver = match_maker_event_sender.clone();


        let mut players = Vec::new();


        // Player 1
        let player_name: PlayerName = PlayerName::TiltInventory;
        players.push(player_name.clone());
        let player_handle: tokio::task::JoinHandle<()> = tokio::task::spawn(async move {
            let mut player: TiltInventory = TiltInventory::new(player_name, false, 4000, 8000, player_1_event_receiver, player_1_order_sender);
            player.start().await;
        });
        handles.push(player_handle);



        // Player 2
        let player_name: PlayerName = PlayerName::Spread;
        players.push(player_name.clone());
        let player_handle: tokio::task::JoinHandle<()> = tokio::task::spawn(async move {
            let mut player: GenericPlayer = GenericPlayer::new(player_name, false, 4000, 8000, player_2_event_receiver, player_2_order_sender);
            player.start().await;
        });
        handles.push(player_handle);


        // Player 3
        let player_name: PlayerName = PlayerName::Seller;
        players.push(player_name.clone());
        let player_handle: tokio::task::JoinHandle<()> = tokio::task::spawn(async move {
            let mut player: GenericPlayer = GenericPlayer::new(player_name, false, 4000, 8000, player_3_event_receiver, player_3_order_sender);
            player.start().await;
        });
        handles.push(player_handle);


        // Player 4
        let player_name: PlayerName = PlayerName::Noisy;
        players.push(player_name.clone());
        let player_handle: tokio::task::JoinHandle<()> = tokio::task::spawn(async move {
            let mut player: GenericPlayer = GenericPlayer::new(player_name, false, 4000, 8000, player_4_event_receiver, player_4_order_sender);
            player.start().await;
        });
        handles.push(player_handle);


        // Player 5
        let player_name: PlayerName = PlayerName::PickOff;
        players.push(player_name.clone());
        let player_handle: tokio::task::JoinHandle<()> = tokio::task::spawn(async move {
            let mut player: EventDrivenPlayer = EventDrivenPlayer::new(player_name, true, player_5_event_receiver, player_5_order_sender);
            player.start().await;
        });
        handles.push(player_handle);



        // Matchmaker
        let match_maker_handle: tokio::task::JoinHandle<()> = tokio::task::spawn(async move {
            let mut match_maker: MatchMaker = MatchMaker::new(STARTING_BALANCE, players, match_maker_event_sender, match_maker_order_receiver);
            match_maker.start().await;
        });
        handles.push(match_maker_handle);



        for handle in handles {
            handle.await.unwrap();
        }

    });

}