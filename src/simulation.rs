// // simulation.rs - Trading bot simulation and strategies

// use rand::Rng;
// use crate::types::{Asset, TraderAction};
// use crate::market::Market;

// /// Represents a trading bot with a specific strategy
// pub struct TraderBot {
//     pub address: String,
//     pub strategy: String,
// }

// impl TraderBot {
//     /// Create a new trading bot
//     pub fn new(address: String, strategy: String) -> Self {
//         TraderBot { address, strategy }
//     }

//     /// Decide what action to take based on the bot's strategy
//     pub fn decide_action(&self, market: &Market) -> TraderAction {
//         let mut rng = rand::thread_rng();
        
//         match self.strategy.as_str() {
//             "aggressive_seller" => {
//                 self.aggressive_seller_strategy(&mut rng, market)
//             },
//             "aggressive_buyer" => {
//                 self.aggressive_buyer_strategy(&mut rng, market)
//             },
//             "balanced" => {
//                 self.balanced_strategy(&mut rng, market)
//             },
//             "market_maker" => {
//                 self.market_maker_strategy(&mut rng, market)
//             },
//             _ => TraderAction::DoNothing,
//         }
//     }

//     /// Strategy that focuses on selling options frequently
//     fn aggressive_seller_strategy(&self, rng: &mut impl Rng, _market: &Market) -> TraderAction {
//         if rng.gen_bool(0.7) {
//             let assets = [Asset::BTC, Asset::ETH, Asset::SOL, Asset::APPLE];
//             let asset = assets[rng.gen_range(0..assets.len())].clone();
//             let strike_price = rng.gen_range(50.0..200.0);
//             let ask_price = rng.gen_range(1.0..10.0);
            
//             if rng.gen_bool(0.6) {
//                 TraderAction::ListCall(asset, strike_price, ask_price)
//             } else {
//                 TraderAction::ListPut(asset, strike_price, ask_price)
//             }
//         } else {
//             TraderAction::DoNothing
//         }
//     }

//     /// Strategy that focuses on buying options frequently
//     fn aggressive_buyer_strategy(&self, rng: &mut impl Rng, market: &Market) -> TraderAction {
//         if rng.gen_bool(0.8) && !market.listings.is_empty() {
//             let listing_ids: Vec<i64> = market.listings.keys().cloned().collect();
//             let random_listing = listing_ids[rng.gen_range(0..listing_ids.len())];
//             TraderAction::BuyOption(random_listing)
//         } else {
//             TraderAction::DoNothing
//         }
//     }

//     /// Balanced strategy between buying and selling
//     fn balanced_strategy(&self, rng: &mut impl Rng, market: &Market) -> TraderAction {
//         let action_type = rng.gen_range(0..3);
//         match action_type {
//             0 => {
//                 // List option
//                 let assets = [Asset::BTC, Asset::ETH, Asset::SOL, Asset::APPLE];
//                 let asset = assets[rng.gen_range(0..assets.len())].clone();
//                 let strike_price = rng.gen_range(50.0..200.0);
//                 let ask_price = rng.gen_range(1.0..10.0);
                
//                 if rng.gen_bool(0.5) {
//                     TraderAction::ListCall(asset, strike_price, ask_price)
//                 } else {
//                     TraderAction::ListPut(asset, strike_price, ask_price)
//                 }
//             },
//             1 => {
//                 // Buy option
//                 if !market.listings.is_empty() {
//                     let listing_ids: Vec<i64> = market.listings.keys().cloned().collect();
//                     let random_listing = listing_ids[rng.gen_range(0..listing_ids.len())];
//                     TraderAction::BuyOption(random_listing)
//                 } else {
//                     TraderAction::DoNothing
//                 }
//             },
//             _ => TraderAction::DoNothing,
//         }
//     }

//     /// Market maker strategy - provides liquidity by maintaining both buy and sell orders
//     fn market_maker_strategy(&self, rng: &mut impl Rng, market: &Market) -> TraderAction {
//         // Market makers try to profit from bid-ask spreads
//         // They typically list options at competitive prices
//         if rng.gen_bool(0.9) {
//             let assets = [Asset::BTC, Asset::ETH, Asset::SOL, Asset::APPLE];
//             let asset = assets[rng.gen_range(0..assets.len())].clone();
            
//             // Use tighter spreads and more competitive pricing
//             let strike_price = rng.gen_range(80.0..150.0);
//             let ask_price = rng.gen_range(0.5..5.0); // Lower ask prices for market making
            
//             if rng.gen_bool(0.5) {
//                 TraderAction::ListCall(asset, strike_price, ask_price)
//             } else {
//                 TraderAction::ListPut(asset, strike_price, ask_price)
//             }
//         } else if !market.listings.is_empty() && rng.gen_bool(0.3) {
//             // Occasionally buy underpriced options
//             let listing_ids: Vec<i64> = market.listings.keys().cloned().collect();
//             let random_listing = listing_ids[rng.gen_range(0..listing_ids.len())];
//             TraderAction::BuyOption(random_listing)
//         } else {
//             TraderAction::DoNothing
//         }
//     }
// }

// /// Run a market simulation with multiple trading bots
// pub fn run_simulation(rounds: usize, verbose: bool) {
//     println!("🚀 Starting Options Trading Market Simulation");
    
//     // Initialize market
//     let mut market = Market::new();
    
//     // Create users with different strategies and capital
//     let user_configs = vec![
//         ("alice", 10000.0, "aggressive_seller"),
//         ("bob", 15000.0, "aggressive_buyer"),
//         ("charlie", 12000.0, "balanced"),
//         ("diana", 8000.0, "market_maker"),
//         ("eve", 20000.0, "aggressive_seller"),
//     ];
    
//     // Create trader bots
//     let mut bots: Vec<TraderBot> = Vec::new();
    
//     for (name, initial_cash, strategy) in user_configs {
//         let user = crate::user::User::new(name.to_string(), initial_cash);
//         let address = user.address.clone();
//         market.add_user(user);
//         bots.push(TraderBot::new(address, strategy.to_string()));
//     }
    
//     println!("Created {} traders", bots.len());
//     if verbose {
//         market.display_users();
//     }
    
//     // Run simulation
//     for round in 1..=rounds {
//         if verbose {
//             println!("\n🔄 Round {} of {}", round, rounds);
//         }
        
//         // Each bot takes an action
//         for bot in &bots {
//             let action = bot.decide_action(&market);
//             execute_action(&mut market, bot, action, verbose);
//         }
        
//         // Display market state periodically
//         if verbose && round % 5 == 0 {
//             market.display_listings();
//             market.display_users();
            
//             let stats = market.get_market_stats();
//             println!("📊 Market Stats: {} listings ({} calls, {} puts), Total value: ${:.2}", 
//                     stats.total_listings, stats.call_count, stats.put_count, stats.total_premium_value);
//         }
        
//         // Add delay for readability
//         if verbose {
//             std::thread::sleep(std::time::Duration::from_millis(200));
//         }
//     }
    
//     println!("\n🏁 Simulation Complete!");
//     market.display_listings();
//     market.display_users();
    
//     let final_stats = market.get_market_stats();
//     println!("📊 Final Market Stats: {} listings, Total value: ${:.2}", 
//             final_stats.total_listings, final_stats.total_premium_value);
// }

// /// Execute a trader action
// fn execute_action(market: &mut Market, bot: &TraderBot, action: TraderAction, verbose: bool) {
//     use chrono::Utc;
//     use crate::types::ListingType;
    
//     match action {
//         TraderAction::ListCall(asset, strike_price, ask_price) => {
//             let expiration = Utc::now() + chrono::Duration::days(30);
//             match market.list_option(asset, strike_price, expiration, ask_price, ListingType::CALL, bot.address.clone()) {
//                 Ok(_) => {},
//                 Err(e) => {
//                     if verbose {
//                         println!("❌ {} failed to list CALL: {}", bot.address, e);
//                     }
//                 },
//             }
//         },
//         TraderAction::ListPut(asset, strike_price, ask_price) => {
//             let expiration = Utc::now() + chrono::Duration::days(30);
//             match market.list_option(asset, strike_price, expiration, ask_price, ListingType::PUT, bot.address.clone()) {
//                 Ok(_) => {},
//                 Err(e) => {
//                     if verbose {
//                         println!("❌ {} failed to list PUT: {}", bot.address, e);
//                     }
//                 },
//             }
//         },
//         TraderAction::BuyOption(listing_id) => {
//             match market.buy_option(listing_id, bot.address.clone()) {
//                 Ok(_) => {},
//                 Err(e) => {
//                     if verbose {
//                         println!("❌ {} failed to buy option {}: {}", bot.address, listing_id, e);
//                     }
//                 },
//             }
//         },
//         TraderAction::DoNothing => {
//             if verbose {
//                 println!("💤 {} is taking a break this round", bot.address);
//             }
//         },
//     }
// }
