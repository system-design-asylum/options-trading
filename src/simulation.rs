// simulation.rs - Enhanced Trading bot simulation with spot trading and dynamic rates

use crate::exchange::SpotAction;
use crate::exchange_rate_provider::{
    default_exchange_rate_provider_admin_address, get_rate_provider,
};
use crate::{Address, Asset, Exchange, ListingOption, ListingType, User};
use chrono::{Duration, Utc};
use rand::Rng;

/// Actions that a trading bot can take
#[derive(Debug, Clone)]
pub enum TraderAction {
    ListCall(Asset, f64, f64), // asset, strike_price, ask_price
    ListPut(Asset, f64, f64),  // asset, strike_price, ask_price
    BuyOption(u32),            // listing_id
    ExerciseOption(u32),       // listing_id
    SpotBuy(Asset, f64),       // asset, amount
    SpotSell(Asset, f64),      // asset, amount
    DoNothing,
}

/// PnL tracking for comprehensive profit/loss analysis
#[derive(Debug, Clone)]
pub struct PnLTracker {
    pub trader_address: Address,
    pub trader_name: String,
    pub initial_portfolio_value: f64,
    pub options_premium_received: f64,
    pub options_premium_paid: f64,
    pub options_exercise_pnl: f64,
    pub spot_trading_pnl: f64,
    pub current_portfolio_value: f64,
    pub trades_log: Vec<TradeRecord>,
}

#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub round: u32,
    pub trade_type: String,
    pub asset: String,
    pub amount: f64,
    pub price: f64,
    pub pnl: f64,
    pub description: String,
}

impl PnLTracker {
    pub fn new(address: Address, name: String, initial_value: f64) -> Self {
        PnLTracker {
            trader_address: address,
            trader_name: name,
            initial_portfolio_value: initial_value,
            options_premium_received: 0.0,
            options_premium_paid: 0.0,
            options_exercise_pnl: 0.0,
            spot_trading_pnl: 0.0,
            current_portfolio_value: initial_value,
            trades_log: Vec::new(),
        }
    }

    pub fn record_trade(&mut self, record: TradeRecord) {
        self.trades_log.push(record);
    }

    pub fn total_pnl(&self) -> f64 {
        self.current_portfolio_value - self.initial_portfolio_value
    }

    pub fn options_pnl(&self) -> f64 {
        self.options_premium_received - self.options_premium_paid + self.options_exercise_pnl
    }
}

/// Market volatility simulation data
pub struct MarketVolatility {
    pub btc_trend: f64, // Current trend factor
    pub eth_trend: f64,
    pub sol_trend: f64,
    pub apple_trend: f64,
    pub volatility_factor: f64, // Overall market volatility
    // Base rates for calculations
    pub base_btc_rate: f64,
    pub base_eth_rate: f64,
    pub base_sol_rate: f64,
    pub base_apple_rate: f64,
}

impl MarketVolatility {
    pub fn new() -> Self {
        MarketVolatility {
            btc_trend: 1.0,
            eth_trend: 1.0,
            sol_trend: 1.0,
            apple_trend: 1.0,
            volatility_factor: 0.05, // 5% standard volatility
            base_btc_rate: 45000.0,
            base_eth_rate: 3000.0,
            base_sol_rate: 100.0,
            base_apple_rate: 150.0,
        }
    }

    /// Update market conditions with random price movements - BALANCED VERSION
    pub fn update_market(&mut self, rng: &mut impl Rng) {
        // Create dramatic market events occasionally with balanced probabilities
        let market_event = rng.gen_range(0.0..1.0);

        if market_event < 0.08 {
            // 8% chance of major bullish event (increased from bear events)
            self.volatility_factor = rng.gen_range(0.15..0.35); // High volatility
            println!("�� MASSIVE BULL RUN! Markets exploding upward!");
        } else if market_event < 0.12 {
            // 4% chance of major bearish event (decreased)
            self.volatility_factor = rng.gen_range(0.15..0.35); // High volatility  
            println!("[CRASH] Massive sell-off!");
        } else if market_event < 0.25 {
            // 13% chance of moderate bullish momentum
            self.volatility_factor = rng.gen_range(0.08..0.15);
            println!("� Strong bullish momentum building");
        } else if market_event < 0.35 {
            // 10% chance of moderate bearish pressure
            self.volatility_factor = rng.gen_range(0.08..0.15);
            println!("[BEARISH] Market experiencing correction pressure");
        } else {
            self.volatility_factor = rng.gen_range(0.02..0.08); // Normal volatility
        }

        // Update individual asset trends with positive bias and mean reversion toward growth
        self.btc_trend = self.update_trend_balanced(self.btc_trend, rng, 1.05); // 5% growth bias
        self.eth_trend = self.update_trend_balanced(self.eth_trend, rng, 1.04); // 4% growth bias
        self.sol_trend = self.update_trend_balanced(self.sol_trend, rng, 1.06); // 6% growth bias
        self.apple_trend = self.update_trend_balanced(self.apple_trend, rng, 1.02); // 2% growth bias
    }

    /// Balanced trend update with growth bias - more realistic market dynamics
    fn update_trend_balanced(&self, current_trend: f64, rng: &mut impl Rng, growth_target: f64) -> f64 {
        // Mean reversion toward growth target instead of 1.0
        let mean_reversion = (growth_target - current_trend) * 0.05; // Slower reversion
        
        // Random walk with slight upward bias
        let random_change = (rng.gen_range(0.0..1.0) - 0.45) * self.volatility_factor * 2.0; // 0.45 vs 0.5 = upward bias
        
        let new_trend = current_trend + mean_reversion + random_change;
        
        // More generous ranges allowing for bigger gains and more moderate losses
        new_trend.max(0.6).min(3.5) // 40% max loss, 250% max gain
    }

    fn update_trend(&self, current_trend: f64, rng: &mut impl Rng) -> f64 {
        // Mean reversion: trends tend to move back toward 1.0
        let mean_reversion = (1.0 - current_trend) * 0.1;
        let random_change = (rng.gen_range(0.0..1.0) - 0.5) * self.volatility_factor * 2.0;

        (current_trend + mean_reversion + random_change)
            .max(0.3)
            .min(3.0)
    }

    /// Get updated price for an asset
    pub fn get_new_price(&self, asset: &Asset, base_price: f64, rng: &mut impl Rng) -> f64 {
        let trend = match asset {
            Asset::BTC => self.btc_trend,
            Asset::ETH => self.eth_trend,
            Asset::SOL => self.sol_trend,
            Asset::APPLE => self.apple_trend,
            Asset::USDT | Asset::USDC | Asset::VNDT | Asset::VNDC => return base_price, // Stable coins
            Asset::OTHER(_) => 1.0, // Unknown assets get neutral trend
        };

        let noise = (rng.gen_range(0.0..1.0) - 0.5) * self.volatility_factor;
        (base_price * trend * (1.0 + noise)).max(1.0)
    }
}

/// Represents a trading bot with a specific strategy
pub struct TraderBot {
    pub address: Address,
    pub strategy: String,
    pub pnl_tracker: PnLTracker,
    pub last_exercise_round: Option<u32>, // Track when last option was exercised
}

impl TraderBot {
    /// Create a new trading bot
    pub fn new(address: Address, strategy: String) -> Self {
        let name = format!("Trader_{}", &address.to_string()[2..8]);
        TraderBot {
            pnl_tracker: PnLTracker::new(address.clone(), name, 0.0),
            address,
            strategy,
            last_exercise_round: None,
        }
    }

    /// Decide what action to take based on the bot's strategy
    pub fn decide_action(&self, exchange: &Exchange, current_round: u32) -> TraderAction {
        let mut rng = rand::thread_rng();

        // Check if we recently exercised an option (within 2 rounds) and should consider spot trading
        if let Some(last_exercise) = self.last_exercise_round {
            if current_round - last_exercise <= 2 && rng.gen_bool(0.7) {
                // 70% chance to make a spot trade after exercising to realize profits
                return self.post_exercise_spot_trade(&mut rng, exchange);
            }
        }

        match self.strategy.as_str() {
            "aggressive_seller" => self.aggressive_seller_strategy(&mut rng, exchange),
            "aggressive_buyer" => self.aggressive_buyer_strategy(&mut rng, exchange),
            "balanced" => self.balanced_strategy(&mut rng, exchange),
            "market_maker" => self.market_maker_strategy(&mut rng, exchange),
            "arbitrageur" => self.arbitrageur_strategy(&mut rng, exchange),
            "momentum_trader" => self.momentum_trader_strategy(&mut rng, exchange),
            "contrarian" => self.contrarian_strategy(&mut rng, exchange),
            "scalper" => self.scalper_strategy(&mut rng, exchange),
            "whale" => self.whale_strategy(&mut rng, exchange),
            _ => TraderAction::DoNothing,
        }
    }

    /// Strategy that focuses on selling options frequently and some spot trading
    fn aggressive_seller_strategy(&self, rng: &mut impl Rng, exchange: &Exchange) -> TraderAction {
        let action_type = rng.gen_range(0.0..1.0);

        if action_type < 0.6 {
            // 60% option listing
            let assets = [Asset::BTC, Asset::ETH, Asset::SOL, Asset::APPLE];
            let asset = assets[rng.gen_range(0..assets.len())].clone();
            let strike_price = rng.gen_range(1000.0..100000.0);
            let ask_price = rng.gen_range(10.0..500.0);

            if rng.gen_bool(0.6) {
                TraderAction::ListCall(asset, strike_price, ask_price)
            } else {
                TraderAction::ListPut(asset, strike_price, ask_price)
            }
        } else if action_type < 0.8 {
            // 20% spot selling with balance check
            let assets = [Asset::BTC, Asset::ETH, Asset::SOL];
            let asset = assets[rng.gen_range(0..assets.len())].clone();

            // Check user's balance and sell a reasonable percentage
            if let Some(user) = exchange.users.get(&self.address) {
                let user_balance = user.get_balance(&asset);
                if user_balance > 0.1 {
                    let max_sellable = user_balance * 0.5; // Sell up to 50% of holdings
                    let amount = (rng.gen_range(0.1..1.0) * max_sellable).min(2.0);
                    TraderAction::SpotSell(asset, amount)
                } else {
                    TraderAction::DoNothing
                }
            } else {
                TraderAction::DoNothing
            }
        } else {
            TraderAction::DoNothing
        }
    }

    /// Strategy that focuses on buying options and spot trading
    fn aggressive_buyer_strategy(&self, rng: &mut impl Rng, exchange: &Exchange) -> TraderAction {
        let action_type = rng.gen_range(0.0..1.0);

        if action_type < 0.5 && !exchange.listings.is_empty() {
            // 50% option buying
            let listing_ids: Vec<u32> = exchange.listings.keys().cloned().collect();
            let random_listing = listing_ids[rng.gen_range(0..listing_ids.len())];
            TraderAction::BuyOption(random_listing)
        } else if action_type < 0.8 {
            // 30% spot buying with balance check
            let assets = [Asset::BTC, Asset::ETH, Asset::SOL];
            let asset = assets[rng.gen_range(0..assets.len())].clone();

            // Check USDT balance for purchasing
            if let Some(user) = exchange.users.get(&self.address) {
                let usdt_balance = user.get_balance(&Asset::USDT);
                if usdt_balance > 1000.0 {
                    // Need at least 1000 USDT to trade
                    let max_trade_value = (usdt_balance * 0.1).min(10000.0); // Use up to 10% of USDT, max 10k
                    // Get current price to calculate amount
                    let rate_provider = crate::exchange_rate_provider::get_readonly_rate_provider();
                    if let Some(price) = rate_provider.get_rate(&asset, &Asset::USDT) {
                        let amount = (max_trade_value / price) * rng.gen_range(0.1..1.0);
                        TraderAction::SpotBuy(asset, amount)
                    } else {
                        TraderAction::DoNothing
                    }
                } else {
                    TraderAction::DoNothing
                }
            } else {
                TraderAction::DoNothing
            }
        } else {
            TraderAction::DoNothing
        }
    }

    /// Balanced strategy between buying and selling options and spot trading
    fn balanced_strategy(&self, rng: &mut impl Rng, exchange: &Exchange) -> TraderAction {
        let action_type = rng.gen_range(0..6);
        match action_type {
            0 => {
                // List option
                let assets = [Asset::BTC, Asset::ETH, Asset::SOL, Asset::APPLE];
                let asset = assets[rng.gen_range(0..assets.len())].clone();
                let strike_price = rng.gen_range(1000.0..100000.0);
                let ask_price = rng.gen_range(10.0..500.0);

                if rng.gen_bool(0.5) {
                    TraderAction::ListCall(asset, strike_price, ask_price)
                } else {
                    TraderAction::ListPut(asset, strike_price, ask_price)
                }
            }
            1 => {
                // Buy option
                if !exchange.listings.is_empty() {
                    let listing_ids: Vec<u32> = exchange.listings.keys().cloned().collect();
                    let random_listing = listing_ids[rng.gen_range(0..listing_ids.len())];
                    TraderAction::BuyOption(random_listing)
                } else {
                    TraderAction::DoNothing
                }
            }
            2 => {
                // Exercise option (if user has any purchased options)
                if !exchange.listings.is_empty() {
                    let exercisable_options: Vec<u32> = exchange
                        .listings
                        .iter()
                        .filter_map(|(id, listing)| {
                            if listing.beneficiary_address.as_ref() == Some(&self.address)
                                && listing.is_purchased
                                && !listing.is_exercised
                            {
                                Some(*id)
                            } else {
                                None
                            }
                        })
                        .collect();

                    if !exercisable_options.is_empty() && rng.gen_bool(0.3) {
                        let random_option =
                            exercisable_options[rng.gen_range(0..exercisable_options.len())];
                        TraderAction::ExerciseOption(random_option)
                    } else {
                        TraderAction::DoNothing
                    }
                } else {
                    TraderAction::DoNothing
                }
            }
            3 => {
                // Spot buy with balance check
                let assets = [Asset::BTC, Asset::ETH, Asset::SOL];
                let asset = assets[rng.gen_range(0..assets.len())].clone();

                if let Some(user) = exchange.users.get(&self.address) {
                    let usdt_balance = user.get_balance(&Asset::USDT);
                    if usdt_balance > 2000.0 {
                        let rate_provider =
                            crate::exchange_rate_provider::get_readonly_rate_provider();
                        if let Some(price) = rate_provider.get_rate(&asset, &Asset::USDT) {
                            let trade_value = usdt_balance * 0.15; // Use 15% of USDT
                            let amount = (trade_value / price) * rng.gen_range(0.3..1.0);
                            TraderAction::SpotBuy(asset, amount)
                        } else {
                            TraderAction::DoNothing
                        }
                    } else {
                        TraderAction::DoNothing
                    }
                } else {
                    TraderAction::DoNothing
                }
            }
            4 => {
                // Spot sell with balance check
                let assets = [Asset::BTC, Asset::ETH, Asset::SOL];
                let asset = assets[rng.gen_range(0..assets.len())].clone();

                if let Some(user) = exchange.users.get(&self.address) {
                    let asset_balance = user.get_balance(&asset);
                    if asset_balance > 0.2 {
                        let amount = (asset_balance * 0.3).min(2.0);
                        TraderAction::SpotSell(asset, amount)
                    } else {
                        TraderAction::DoNothing
                    }
                } else {
                    TraderAction::DoNothing
                }
            }
            _ => TraderAction::DoNothing,
        }
    }

    /// Post-exercise spot trading to realize profits
    fn post_exercise_spot_trade(&self, rng: &mut impl Rng, exchange: &Exchange) -> TraderAction {
        let assets = [Asset::BTC, Asset::ETH, Asset::SOL, Asset::APPLE];
        let asset = assets[rng.gen_range(0..assets.len())].clone();

        // After exercising options, traders often want to realize profits by selling assets
        // or reinvest by buying more assets
        if rng.gen_bool(0.6) {
            // 60% chance to sell (realize profits)
            if let Some(user) = exchange.users.get(&self.address) {
                let asset_balance = user.get_balance(&asset);
                if asset_balance > 0.1 {
                    let amount = (asset_balance * 0.4).min(3.0); // Sell up to 40% after exercise
                    TraderAction::SpotSell(asset, amount)
                } else {
                    TraderAction::DoNothing
                }
            } else {
                TraderAction::DoNothing
            }
        } else {
            // 40% chance to buy (reinvest)
            if let Some(user) = exchange.users.get(&self.address) {
                let usdt_balance = user.get_balance(&Asset::USDT);
                if usdt_balance > 5000.0 {
                    let rate_provider = crate::exchange_rate_provider::get_readonly_rate_provider();
                    if let Some(price) = rate_provider.get_rate(&asset, &Asset::USDT) {
                        let reinvest_amount = usdt_balance * 0.2; // Reinvest 20% of USDT
                        let amount = (reinvest_amount / price) * rng.gen_range(0.5..1.0);
                        TraderAction::SpotBuy(asset, amount)
                    } else {
                        TraderAction::DoNothing
                    }
                } else {
                    TraderAction::DoNothing
                }
            } else {
                TraderAction::DoNothing
            }
        }
    }

    /// Market maker strategy - provides liquidity by maintaining both buy and sell orders
    fn market_maker_strategy(&self, rng: &mut impl Rng, exchange: &Exchange) -> TraderAction {
        // Market makers try to profit from bid-ask spreads
        // They typically list options at competitive prices
        if rng.gen_bool(0.9) {
            let assets = [Asset::BTC, Asset::ETH, Asset::SOL, Asset::APPLE];
            let asset = assets[rng.gen_range(0..assets.len())].clone();

            // Use tighter spreads and more competitive pricing
            let strike_price = rng.gen_range(30000.0..80000.0);
            let ask_price = rng.gen_range(5.0..200.0); // Lower ask prices for market making

            if rng.gen_bool(0.5) {
                TraderAction::ListCall(asset, strike_price, ask_price)
            } else {
                TraderAction::ListPut(asset, strike_price, ask_price)
            }
        } else if !exchange.listings.is_empty() && rng.gen_bool(0.3) {
            // Occasionally buy underpriced options
            let listing_ids: Vec<u32> = exchange.listings.keys().cloned().collect();
            let random_listing = listing_ids[rng.gen_range(0..listing_ids.len())];
            TraderAction::BuyOption(random_listing)
        } else {
            TraderAction::DoNothing
        }
    }

    /// Arbitrageur strategy - looks for price discrepancies and exercises profitable options
    fn arbitrageur_strategy(&self, rng: &mut impl Rng, exchange: &Exchange) -> TraderAction {
        // First priority: exercise profitable options
        let exercisable_options: Vec<u32> = exchange
            .listings
            .iter()
            .filter_map(|(id, listing)| {
                if listing.beneficiary_address.as_ref() == Some(&self.address)
                    && listing.is_purchased
                    && !listing.is_exercised
                {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        if !exercisable_options.is_empty() && rng.gen_bool(0.8) {
            let random_option = exercisable_options[rng.gen_range(0..exercisable_options.len())];
            return TraderAction::ExerciseOption(random_option);
        }

        // Second priority: spot arbitrage between assets
        if rng.gen_bool(0.6) {
            let assets = [Asset::BTC, Asset::ETH, Asset::SOL];
            let asset = assets[rng.gen_range(0..assets.len())].clone();

            if let Some(user) = exchange.users.get(&self.address) {
                let user_balance = user.get_balance(&asset);
                if user_balance > 0.1 {
                    let amount = (user_balance * 0.3).min(1.0);
                    TraderAction::SpotSell(asset, amount)
                } else {
                    TraderAction::DoNothing
                }
            } else {
                TraderAction::DoNothing
            }
        } else {
            TraderAction::DoNothing
        }
    }

    /// Momentum trader - follows market trends
    fn momentum_trader_strategy(&self, rng: &mut impl Rng, exchange: &Exchange) -> TraderAction {
        let action_type = rng.gen_range(0.0..1.0);

        if action_type < 0.4 && !exchange.listings.is_empty() {
            // Buy options aggressively during momentum
            let listing_ids: Vec<u32> = exchange.listings.keys().cloned().collect();
            let random_listing = listing_ids[rng.gen_range(0..listing_ids.len())];
            TraderAction::BuyOption(random_listing)
        } else if action_type < 0.7 {
            // Spot trading following momentum
            let assets = [Asset::BTC, Asset::ETH, Asset::SOL];
            let asset = assets[rng.gen_range(0..assets.len())].clone();

            if let Some(user) = exchange.users.get(&self.address) {
                let usdt_balance = user.get_balance(&Asset::USDT);
                if usdt_balance > 5000.0 {
                    let rate_provider = crate::exchange_rate_provider::get_readonly_rate_provider();
                    if let Some(price) = rate_provider.get_rate(&asset, &Asset::USDT) {
                        let trade_value = usdt_balance * 0.2; // Use 20% of USDT
                        let amount = (trade_value / price) * rng.gen_range(0.5..1.0);
                        TraderAction::SpotBuy(asset, amount)
                    } else {
                        TraderAction::DoNothing
                    }
                } else {
                    TraderAction::DoNothing
                }
            } else {
                TraderAction::DoNothing
            }
        } else {
            TraderAction::DoNothing
        }
    }

    /// Contrarian strategy - goes against market trends
    fn contrarian_strategy(&self, rng: &mut impl Rng, exchange: &Exchange) -> TraderAction {
        let action_type = rng.gen_range(0.0..1.0);

        if action_type < 0.5 {
            // List conservative options during market volatility
            let assets = [Asset::BTC, Asset::ETH, Asset::SOL, Asset::APPLE];
            let asset = assets[rng.gen_range(0..assets.len())].clone();
            let strike_price = rng.gen_range(20000.0..60000.0);
            let ask_price = rng.gen_range(100.0..800.0);

            if rng.gen_bool(0.5) {
                TraderAction::ListPut(asset, strike_price, ask_price) // Prefer puts during bullish times
            } else {
                TraderAction::ListCall(asset, strike_price, ask_price)
            }
        } else if action_type < 0.8 {
            // Contrarian spot trading
            let assets = [Asset::BTC, Asset::ETH, Asset::SOL];
            let asset = assets[rng.gen_range(0..assets.len())].clone();

            if let Some(user) = exchange.users.get(&self.address) {
                let asset_balance = user.get_balance(&asset);
                if asset_balance > 0.5 {
                    let amount = (asset_balance * 0.4).min(3.0);
                    TraderAction::SpotSell(asset, amount)
                } else {
                    TraderAction::DoNothing
                }
            } else {
                TraderAction::DoNothing
            }
        } else {
            TraderAction::DoNothing
        }
    }

    /// Scalper strategy - makes frequent small trades
    fn scalper_strategy(&self, rng: &mut impl Rng, exchange: &Exchange) -> TraderAction {
        let action_type = rng.gen_range(0.0..1.0);

        if action_type < 0.3 && !exchange.listings.is_empty() {
            // Quick option trades
            let listing_ids: Vec<u32> = exchange.listings.keys().cloned().collect();
            let random_listing = listing_ids[rng.gen_range(0..listing_ids.len())];
            TraderAction::BuyOption(random_listing)
        } else if action_type < 0.8 {
            // Frequent small spot trades
            let assets = [Asset::BTC, Asset::ETH, Asset::SOL];
            let asset = assets[rng.gen_range(0..assets.len())].clone();

            if rng.gen_bool(0.5) {
                // Small buys
                if let Some(user) = exchange.users.get(&self.address) {
                    let usdt_balance = user.get_balance(&Asset::USDT);
                    if usdt_balance > 2000.0 {
                        let rate_provider =
                            crate::exchange_rate_provider::get_readonly_rate_provider();
                        if let Some(price) = rate_provider.get_rate(&asset, &Asset::USDT) {
                            let small_trade = usdt_balance * 0.05; // Only 5% per trade
                            let amount = (small_trade / price) * rng.gen_range(0.8..1.0);
                            TraderAction::SpotBuy(asset, amount)
                        } else {
                            TraderAction::DoNothing
                        }
                    } else {
                        TraderAction::DoNothing
                    }
                } else {
                    TraderAction::DoNothing
                }
            } else {
                // Small sells
                if let Some(user) = exchange.users.get(&self.address) {
                    let asset_balance = user.get_balance(&asset);
                    if asset_balance > 0.2 {
                        let amount = (asset_balance * 0.1).min(0.5); // Very small amounts
                        TraderAction::SpotSell(asset, amount)
                    } else {
                        TraderAction::DoNothing
                    }
                } else {
                    TraderAction::DoNothing
                }
            }
        } else {
            TraderAction::DoNothing
        }
    }

    /// Whale strategy - makes large, impactful trades
    fn whale_strategy(&self, rng: &mut impl Rng, exchange: &Exchange) -> TraderAction {
        let action_type = rng.gen_range(0.0..1.0);

        if action_type < 0.4 {
            // Large option listings
            let assets = [Asset::BTC, Asset::ETH, Asset::SOL, Asset::APPLE];
            let asset = assets[rng.gen_range(0..assets.len())].clone();
            let strike_price = rng.gen_range(40000.0..120000.0);
            let ask_price = rng.gen_range(500.0..2000.0); // High premiums

            if rng.gen_bool(0.5) {
                TraderAction::ListCall(asset, strike_price, ask_price)
            } else {
                TraderAction::ListPut(asset, strike_price, ask_price)
            }
        } else if action_type < 0.7 && !exchange.listings.is_empty() {
            // Buy multiple options
            let listing_ids: Vec<u32> = exchange.listings.keys().cloned().collect();
            let random_listing = listing_ids[rng.gen_range(0..listing_ids.len())];
            TraderAction::BuyOption(random_listing)
        } else if action_type < 0.9 {
            // Large spot trades that move markets
            let assets = [Asset::BTC, Asset::ETH, Asset::SOL];
            let asset = assets[rng.gen_range(0..assets.len())].clone();

            if rng.gen_bool(0.5) {
                // Large buys
                if let Some(user) = exchange.users.get(&self.address) {
                    let usdt_balance = user.get_balance(&Asset::USDT);
                    if usdt_balance > 50000.0 {
                        let rate_provider =
                            crate::exchange_rate_provider::get_readonly_rate_provider();
                        if let Some(price) = rate_provider.get_rate(&asset, &Asset::USDT) {
                            let whale_trade = usdt_balance * 0.3; // 30% of USDT in one trade
                            let amount = (whale_trade / price) * rng.gen_range(0.7..1.0);
                            TraderAction::SpotBuy(asset, amount)
                        } else {
                            TraderAction::DoNothing
                        }
                    } else {
                        TraderAction::DoNothing
                    }
                } else {
                    TraderAction::DoNothing
                }
            } else {
                // Large sells
                if let Some(user) = exchange.users.get(&self.address) {
                    let asset_balance = user.get_balance(&asset);
                    if asset_balance > 5.0 {
                        let amount = (asset_balance * 0.4).min(10.0); // Large amounts
                        TraderAction::SpotSell(asset, amount)
                    } else {
                        TraderAction::DoNothing
                    }
                } else {
                    TraderAction::DoNothing
                }
            }
        } else {
            TraderAction::DoNothing
        }
    }
}

/// Automatically exercise all profitable options at the end of simulation
fn auto_exercise_profitable_options(
    exchange: &mut Exchange,
    bots: &mut [TraderBot],
    verbose: bool,
) {
    let mut exercised_count = 0;
    let rate_provider = crate::exchange_rate_provider::get_readonly_rate_provider();

    // Get all exercisable options
    let exercisable_options: Vec<(u32, Address)> = exchange
        .listings
        .iter()
        .filter_map(|(id, listing)| {
            if listing.is_purchased && !listing.is_exercised {
                if let Some(beneficiary) = &listing.beneficiary_address {
                    // Check if option is profitable before exercising
                    if let Some(current_price) =
                        rate_provider.get_rate(&listing.base_asset, &Asset::USDT)
                    {
                        let is_profitable = match listing.listing_type {
                            ListingType::CALL => current_price > listing.strike_price,
                            ListingType::PUT => current_price < listing.strike_price,
                        };

                        if is_profitable {
                            Some((*id, beneficiary.clone()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();

    for (option_id, beneficiary_address) in exercisable_options {
        // Find the bot that owns this option
        if let Some(bot) = bots.iter_mut().find(|b| b.address == beneficiary_address) {
            // Check if user has enough balance for option exercise
            if let Some(user) = exchange.users.get(&beneficiary_address) {
                if let Some(listing) = exchange.listings.get(&option_id) {
                    let exercise_cost = listing.strike_price * listing.exercise_amount;
                    let usdt_balance = user.get_balance(&Asset::USDT);

                    // Only proceed if user has enough USDT for exercise
                    if usdt_balance >= exercise_cost {
                        // Try to exercise the option
                        let result =
                            exchange.exercise_option(option_id, beneficiary_address.clone());
                        match result {
                            Ok(_) => {
                                exercised_count += 1;
                                bot.last_exercise_round = Some(1000); // Mark as recently exercised

                                if verbose {
                                    println!("[EXERCISED] {} exercised option #{}", bot.strategy, option_id);
                                }

                                // Post-exercise spot trading with balance validation
                                let mut rng = rand::thread_rng();
                                let post_action = bot.post_exercise_spot_trade(&mut rng, exchange);
                                match post_action {
                                    TraderAction::SpotBuy(asset, amount) => {
                                        // Validate user has enough USDT for the buy
                                        if let Some(updated_user) = exchange.users.get(&bot.address)
                                        {
                                            if let Some(price) =
                                                rate_provider.get_rate(&asset, &Asset::USDT)
                                            {
                                                let trade_cost = amount * price;
                                                let usdt_balance =
                                                    updated_user.get_balance(&Asset::USDT);

                                                if usdt_balance >= trade_cost {
                                                    if let Err(e) = exchange
                                                        .spot_trade_current_price(
                                                            &asset,
                                                            &Asset::USDT,
                                                            amount,
                                                            &SpotAction::BUY,
                                                            bot.address.clone(),
                                                        )
                                                    {
                                                        if verbose {
                                                            println!(
                                                                "[FAILED] {} failed post-exercise spot buy: {}",
                                                                bot.strategy, e
                                                            );
                                                        }
                                                    } else if verbose {
                                                        println!(
                                                            "[POST-EXERCISE] {} made post-exercise spot buy: {:.4} {}",
                                                            bot.strategy, amount, asset
                                                        );
                                                    }
                                                } else if verbose {
                                                    println!(
                                                        "⚠️ {} skipped post-exercise buy: insufficient USDT balance",
                                                        bot.strategy
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    TraderAction::SpotSell(asset, amount) => {
                                        // Validate user has enough of the asset to sell
                                        if let Some(updated_user) = exchange.users.get(&bot.address)
                                        {
                                            let asset_balance = updated_user.get_balance(&asset);
                                            let safe_amount = amount.min(asset_balance);

                                            if safe_amount > 0.001 {
                                                // Only sell if meaningful amount
                                                if let Err(e) = exchange.spot_trade_current_price(
                                                    &asset,
                                                    &Asset::USDT,
                                                    safe_amount,
                                                    &SpotAction::SELL,
                                                    bot.address.clone(),
                                                ) {
                                                    if verbose {
                                                        println!(
                                                            "[FAILED] {} failed post-exercise spot sell: {}",
                                                            bot.strategy, e
                                                        );
                                                    }
                                                } else if verbose {
                                                    println!(
                                                        "[PROFIT] {} realized profits via spot sell: {:.4} {}",
                                                        bot.strategy, safe_amount, asset
                                                    );
                                                }
                                            } else if verbose {
                                                println!(
                                                    "⚠️ {} skipped post-exercise sell: insufficient {} balance",
                                                    bot.strategy, asset
                                                );
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Err(e) => {
                                if verbose {
                                    println!(
                                        "[FAILED] {} failed to exercise option #{}: {}",
                                        bot.strategy, option_id, e
                                    );
                                }
                            }
                        }
                    } else if verbose {
                        println!(
                            "[FAILED] {} failed to exercise option #{}: Insufficient USDT balance",
                            bot.strategy, option_id
                        );
                    }
                }
            }
        }
    }

    if verbose {
        println!(
            "[EXERCISED] Auto-exercised {} profitable options with post-exercise trades!",
            exercised_count
        );
    }
}

/// Run a market simulation with multiple trading bots
pub fn run_simulation(rounds: usize, verbose: bool) {
    println!("🚀 Starting ADVANCED Options Trading Market Simulation");
    println!(
        "💹 Features: Dynamic market volatility, sophisticated trading bots, options exercising, and profit realization"
    );

    // Initialize exchange
    let mut exchange = Exchange::new();

    // Initialize market volatility system
    let mut market_volatility = MarketVolatility::new();

    // Create users with different strategies and initial assets - MORE TRADERS!
    let user_configs = vec![
        ("alice", "aggressive_seller"),
        ("bob", "aggressive_buyer"),
        ("charlie", "balanced"),
        ("diana", "market_maker"),
        ("eve", "aggressive_seller"),
        ("frank", "arbitrageur"),     // New strategy
        ("grace", "momentum_trader"), // New strategy
        ("henry", "contrarian"),      // New strategy
        ("iris", "scalper"),          // New strategy
        ("jack", "whale"),            // New strategy
    ];

    // Create trader bots
    let mut bots: Vec<TraderBot> = Vec::new();

    for (i, (name, strategy)) in user_configs.iter().enumerate() {
        // Create truly unique addresses for each user with different patterns
        let address_str = match i {
            0 => "0x0000000000000000000000000000000000000000".to_string(), // alice
            1 => "0x1234000000000000000000000000000000000001".to_string(), // bob
            2 => "0x1234000000000000000000000000000000000002".to_string(), // charlie
            3 => "0x1234000000000000000000000000000000000003".to_string(), // diana
            4 => "0x1234000000000000000000000000000000000004".to_string(), // eve
            5 => "0x1234000000000000000000000000000000000005".to_string(), // frank
            6 => "0x1234000000000000000000000000000000000006".to_string(), // grace
            7 => "0x1234000000000000000000000000000000000007".to_string(), // henry
            8 => "0x1234000000000000000000000000000000000008".to_string(), // iris
            9 => "0x1234000000000000000000000000000000000009".to_string(), // jack
            _ => format!("0xABCD{:036}", i), // fallback for any additional traders
        };
        let address = Address::from(&address_str).unwrap();

        // Create user with initial assets
        let mut user = User::new(address.clone());

        // Give users initial assets for trading
        user.add_asset(&Asset::USDT, 1000000.0).unwrap(); // 1M USDT
        user.add_asset(&Asset::BTC, 10.0).unwrap(); // 10 BTC
        user.add_asset(&Asset::ETH, 100.0).unwrap(); // 100 ETH
        user.add_asset(&Asset::SOL, 1000.0).unwrap(); // 1000 SOL
        user.add_asset(&Asset::APPLE, 50.0).unwrap(); // 50 APPLE shares

        exchange.users.insert(address.clone(), user);
        let mut bot = TraderBot::new(address, strategy.to_string());

        // Set proper trader name instead of address-based name
        bot.pnl_tracker.trader_name = name.to_string();

        // Initialize PnL tracker with current portfolio value
        if let Some(user) = exchange.users.get(&bot.address) {
            let mut rate_provider = get_rate_provider();
            let initial_value = calculate_portfolio_value(user, &mut rate_provider);
            bot.pnl_tracker.initial_portfolio_value = initial_value;
            bot.pnl_tracker.current_portfolio_value = initial_value;
        }

        bots.push(bot);

        if verbose {
            println!("Created trader {}: {} ({})", i + 1, name, strategy);
        }
    }

    println!("Created {} traders", bots.len());
    if verbose {
        display_users(&exchange);
    }

    // Run simulation
    for round in 1..=rounds {
        // Update market conditions with dynamic exchange rates
        if let Err(e) =
            update_exchange_rates(&exchange, round as u32, &mut market_volatility, verbose)
        {
            eprintln!("Warning: Failed to update exchange rates: {}", e);
        }

        if verbose {
            println!("\n>> Round {} of {}", round, rounds);
        }

        // Each bot takes an action
        for bot in &mut bots {
            let action = bot.decide_action(&exchange, round as u32);
            execute_action(&mut exchange, bot, action, round as u32, verbose);
        }

        // Display market state periodically
        if verbose && round % 5 == 0 {
            display_listings(&exchange);
            display_users(&exchange);

            let stats = get_market_stats(&exchange);
            println!(
                "Market Stats: {} listings ({} calls, {} puts), Total value: ${:.2}",
                stats.total_listings, stats.call_count, stats.put_count, stats.total_premium_value
            );
        }

        // Add delay for readability
        if verbose {
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    println!("\n[SIMULATION COMPLETE]");

    // Exercise profitable options automatically at the end
    if verbose {
        println!("\n[AUTO-EXERCISING PROFITABLE OPTIONS]...");
    }
    auto_exercise_profitable_options(&mut exchange, &mut bots, verbose);

    display_listings(&exchange);
    display_users(&exchange);

    let final_stats = get_market_stats(&exchange);
    println!(
        "Final Market Stats: {} listings, Total value: ${:.2}",
        final_stats.total_listings, final_stats.total_premium_value
    );

    // Generate comprehensive PnL report
    generate_pnl_report(&bots, &exchange);
}

/// Market statistics for display
pub struct MarketStats {
    pub total_listings: usize,
    pub call_count: usize,
    pub put_count: usize,
    pub total_premium_value: f64,
}

/// Get market statistics
fn get_market_stats(exchange: &Exchange) -> MarketStats {
    let total_listings = exchange.listings.len();
    let mut call_count = 0;
    let mut put_count = 0;
    let mut total_premium_value = 0.0;

    for listing in exchange.listings.values() {
        match listing.listing_type {
            ListingType::CALL => call_count += 1,
            ListingType::PUT => put_count += 1,
        }
        total_premium_value += listing.get_premium_price();
    }

    MarketStats {
        total_listings,
        call_count,
        put_count,
        total_premium_value,
    }
}

/// Helper function to display address in a readable format
fn format_address(address: &Address) -> String {
    let addr_str = address.to_string();
    let start = &addr_str[0..6];
    let end = &addr_str[addr_str.len() - 4..];
    format!("{}...{}", start, end)
}

/// Get user name by address (for display purposes)
fn get_user_name(address: &Address) -> &'static str {
    let addr_str = address.to_string();
    match addr_str.chars().last() {
        Some('0') => "alice",
        Some('1') => "bob",
        Some('2') => "charlie",
        Some('3') => "diana",
        Some('4') => "eve",
        _ => "unknown",
    }
}

/// Display current listings
fn display_listings(exchange: &Exchange) {
    println!("\nCurrent Listings:");
    if exchange.listings.is_empty() {
        println!("  No active listings");
        return;
    }

    for (id, listing) in &exchange.listings {
        let status = if listing.is_exercised {
            "EXERCISED"
        } else if listing.is_purchased {
            "PURCHASED"
        } else if listing.is_unlisted {
            "UNLISTED"
        } else {
            "ACTIVE"
        };

        println!(
            "  #{}: {} {} {}/{} @ ${:.2} (strike: ${:.2}) [{}]",
            id,
            listing.listing_type,
            listing.exercise_amount,
            listing.base_asset,
            listing.quote_asset,
            listing.ask_price,
            listing.strike_price,
            status
        );
    }
}

/// Display user balances
fn display_users(exchange: &Exchange) {
    println!("\nUser Balances:");
    for (address, user) in &exchange.users {
        let user_name = get_user_name(address);
        let addr_display = format_address(address);
        println!("  {} ({}): ", user_name, addr_display);
        for (asset, balance) in &user.balances {
            if *balance > 0.0 {
                println!("    {}: {:.2}", asset, balance);
            }
        }
    }
}

/// Execute a trader action
fn execute_action(
    exchange: &mut Exchange,
    bot: &mut TraderBot,
    action: TraderAction,
    round: u32,
    verbose: bool,
) {
    let user_name = get_user_name(&bot.address);
    let addr_display = format_address(&bot.address);

    match action {
        TraderAction::ListCall(base_asset, strike_price, ask_price) => {
            let expiration = Utc::now() + Duration::days(30);
            let bid_price = ask_price * 0.95; // Set bid slightly lower than ask

            let option = ListingOption {
                listing_id: 0, // Will be set by exchange
                base_asset: base_asset.clone(),
                quote_asset: Asset::USDT, // Use USDT as quote asset
                listing_type: ListingType::CALL,
                strike_price,
                ask_price,
                bid_price,
                expiration_time: expiration,
                grantor_address: bot.address.clone(),
                beneficiary_address: None,
                exercise_amount: 1.0, // Default to 1 unit
                is_purchased: false,
                is_unlisted: false,
                is_exercised: false,
            };

            match exchange.list_option(bot.address.clone(), option) {
                Ok(listing_id) => {
                    if verbose {
                        println!(
                            "[LISTED] {} ({}) listed CALL option #{} for {}/{} @ ${:.2}",
                            user_name,
                            addr_display,
                            listing_id,
                            base_asset,
                            Asset::USDT,
                            ask_price
                        );
                    }
                }
                Err(e) => {
                    if verbose {
                        println!(
                            "[FAILED] {} ({}) failed to list CALL: {}",
                            user_name, addr_display, e
                        );
                    }
                }
            }
        }
        TraderAction::ListPut(base_asset, strike_price, ask_price) => {
            let expiration = Utc::now() + Duration::days(30);
            let bid_price = ask_price * 0.95; // Set bid slightly lower than ask

            let option = ListingOption {
                listing_id: 0, // Will be set by exchange
                base_asset: base_asset.clone(),
                quote_asset: Asset::USDT, // Use USDT as quote asset
                listing_type: ListingType::PUT,
                strike_price,
                ask_price,
                bid_price,
                expiration_time: expiration,
                grantor_address: bot.address.clone(),
                beneficiary_address: None,
                exercise_amount: 1.0, // Default to 1 unit
                is_purchased: false,
                is_unlisted: false,
                is_exercised: false,
            };

            match exchange.list_option(bot.address.clone(), option) {
                Ok(listing_id) => {
                    if verbose {
                        println!(
                            "[LISTED] {} ({}) listed PUT option #{} for {}/{} @ ${:.2}",
                            user_name,
                            addr_display,
                            listing_id,
                            base_asset,
                            Asset::USDT,
                            ask_price
                        );
                    }
                }
                Err(e) => {
                    if verbose {
                        println!(
                            "[FAILED] {} ({}) failed to list PUT: {}",
                            user_name, addr_display, e
                        );
                    }
                }
            }
        }
        TraderAction::BuyOption(listing_id) => {
            match exchange.purchase_option(listing_id, bot.address.clone()) {
                Ok(_) => {
                    if verbose {
                        println!(
                            "[PURCHASED] {} ({}) purchased option #{}",
                            user_name, addr_display, listing_id
                        );
                    }
                }
                Err(e) => {
                    if verbose {
                        println!(
                            "[FAILED] {} ({}) failed to buy option {}: {}",
                            user_name, addr_display, listing_id, e
                        );
                    }
                }
            }
        }
        TraderAction::ExerciseOption(listing_id) => {
            match exchange.exercise_option(listing_id, bot.address.clone()) {
                Ok(_) => {
                    // Track that this bot exercised an option for future spot trading
                    bot.last_exercise_round = Some(round);

                    if verbose {
                        println!(
                            "✅ {} ({}) exercised option #{} and may make spot trades to realize profits",
                            user_name, addr_display, listing_id
                        );
                    }
                }
                Err(e) => {
                    if verbose {
                        println!(
                            "❌ {} ({}) failed to exercise option {}: {}",
                            user_name, addr_display, listing_id, e
                        );
                    }
                }
            }
        }
        TraderAction::SpotBuy(asset, amount) => {
            match exchange.spot_trade_current_price(
                &asset,
                &Asset::USDT,
                amount,
                &SpotAction::BUY,
                bot.address.clone(),
            ) {
                Ok(()) => {
                    if verbose {
                        println!(
                            "[BOUGHT] {} ({}) bought {:.2} {} via spot trading",
                            user_name, addr_display, amount, asset
                        );
                    }
                }
                Err(e) => {
                    if verbose {
                        println!(
                            "❌ {} ({}) failed to spot buy {}: {}",
                            user_name, addr_display, asset, e
                        );
                    }
                }
            }
        }
        TraderAction::SpotSell(asset, amount) => {
            match exchange.spot_trade_current_price(
                &asset,
                &Asset::USDT,
                amount,
                &SpotAction::SELL,
                bot.address.clone(),
            ) {
                Ok(()) => {
                    if verbose {
                        println!(
                            "[SOLD] {} ({}) sold {:.2} {} via spot trading",
                            user_name, addr_display, amount, asset
                        );
                    }
                }
                Err(e) => {
                    if verbose {
                        println!(
                            "❌ {} ({}) failed to spot sell {}: {}",
                            user_name, addr_display, asset, e
                        );
                    }
                }
            }
        }
        TraderAction::DoNothing => {
            if verbose {
                println!(
                    "[IDLE] {} ({}) is taking a break this round",
                    user_name, addr_display
                );
            }
        }
    }
}

/// Updates exchange rates with dynamic market behavior
fn update_exchange_rates(
    _exchange: &Exchange,
    round: u32,
    volatility: &mut MarketVolatility,
    verbose: bool,
) -> Result<(), String> {
    // 10% chance of major market events with better balance
    let random_event = rand::random::<f64>();
    let market_event = if random_event < 0.06 {
        // 6% chance of major bullish event
        "🚀 MAJOR BULLISH BREAKOUT! Markets surge dramatically!"
    } else if random_event < 0.10 {
        // 4% chance of major bearish event  
        "[CRASH] Massive sell-off across all assets!"
    } else if random_event < 0.20 {
        // 10% chance of moderate bullish
        "[BULLISH] Market showing strong upward momentum"
    } else if random_event < 0.25 {
        // 5% chance of moderate bearish
        "📉 Market experiencing correction pressure"
    } else {
        ""
    };

    // Update volatility factors with BALANCED probability distribution
    if random_event < 0.06 {
        // Major bullish event - dramatic gains (6% chance)
        volatility.btc_trend *= 1.15 + rand::random::<f64>() * 0.25;  // 15-40% gain
        volatility.eth_trend *= 1.12 + rand::random::<f64>() * 0.20;  // 12-32% gain
        volatility.sol_trend *= 1.18 + rand::random::<f64>() * 0.30;  // 18-48% gain
        volatility.apple_trend *= 1.08 + rand::random::<f64>() * 0.12; // 8-20% gain
    } else if random_event < 0.10 {
        // Major bearish event - but less severe than before (4% chance)
        volatility.btc_trend *= 0.85 - rand::random::<f64>() * 0.10;  // 10-15% loss
        volatility.eth_trend *= 0.88 - rand::random::<f64>() * 0.08;  // 4-12% loss  
        volatility.sol_trend *= 0.80 - rand::random::<f64>() * 0.15;  // 5-20% loss
        volatility.apple_trend *= 0.94 - rand::random::<f64>() * 0.06; // 0-6% loss
    } else if random_event < 0.20 {
        // Moderate bullish momentum (10% chance)
        volatility.btc_trend *= 1.0 + rand::random::<f64>() * 0.08;   // 0-8% gain
        volatility.eth_trend *= 1.0 + rand::random::<f64>() * 0.06;   // 0-6% gain
        volatility.sol_trend *= 1.0 + rand::random::<f64>() * 0.10;   // 0-10% gain
        volatility.apple_trend *= 1.0 + rand::random::<f64>() * 0.04; // 0-4% gain
    } else if random_event < 0.25 {
        // Moderate bearish correction (5% chance) 
        volatility.btc_trend *= 1.0 - rand::random::<f64>() * 0.04;   // 0-4% loss
        volatility.eth_trend *= 1.0 - rand::random::<f64>() * 0.03;   // 0-3% loss
        volatility.sol_trend *= 1.0 - rand::random::<f64>() * 0.05;   // 0-5% loss
        volatility.apple_trend *= 1.0 - rand::random::<f64>() * 0.02; // 0-2% loss
    } else {
        // Normal market movement with POSITIVE bias (75% of time)
        volatility.btc_trend += (1.05 - volatility.btc_trend) * 0.02 + (rand::random::<f64>() - 0.4) * 0.02;
        volatility.eth_trend += (1.04 - volatility.eth_trend) * 0.02 + (rand::random::<f64>() - 0.4) * 0.015;
        volatility.sol_trend += (1.06 - volatility.sol_trend) * 0.03 + (rand::random::<f64>() - 0.4) * 0.025;
        volatility.apple_trend += (1.02 - volatility.apple_trend) * 0.01 + (rand::random::<f64>() - 0.4) * 0.01;
    }

    // More generous clamping ranges - allow bigger gains, limit severe losses
    volatility.btc_trend = volatility.btc_trend.max(0.65).min(3.0);   // Max 35% loss, 200% gain
    volatility.eth_trend = volatility.eth_trend.max(0.70).min(2.8);   // Max 30% loss, 180% gain  
    volatility.sol_trend = volatility.sol_trend.max(0.60).min(3.5);   // Max 40% loss, 250% gain
    volatility.apple_trend = volatility.apple_trend.max(0.80).min(1.8); // Max 20% loss, 80% gain

    // Calculate new rates
    let btc_rate = volatility.base_btc_rate * volatility.btc_trend;
    let eth_rate = volatility.base_eth_rate * volatility.eth_trend;
    let sol_rate = volatility.base_sol_rate * volatility.sol_trend;
    let apple_rate = volatility.base_apple_rate * volatility.apple_trend;

    // Update exchange rates through the rate provider
    let admin_address = default_exchange_rate_provider_admin_address();
    let mut rate_provider = get_rate_provider();

    rate_provider.set_rate(Asset::BTC, Asset::USDT, btc_rate, admin_address.clone())?;
    rate_provider.set_rate(Asset::ETH, Asset::USDT, eth_rate, admin_address.clone())?;
    rate_provider.set_rate(Asset::SOL, Asset::USDT, sol_rate, admin_address.clone())?;
    rate_provider.set_rate(Asset::APPLE, Asset::USDT, apple_rate, admin_address.clone())?;

    // Display market updates
    if verbose && (!market_event.is_empty() || round % 5 == 0) {
        if !market_event.is_empty() {
            println!("\n{}", market_event);
        }
        println!("Market Update (Round {}):", round);
        println!(
            "  BTC: ${:.2} ({:+.1}%)",
            btc_rate,
            (volatility.btc_trend - 1.0) * 100.0
        );
        println!(
            "  ETH: ${:.2} ({:+.1}%)",
            eth_rate,
            (volatility.eth_trend - 1.0) * 100.0
        );
        println!(
            "  SOL: ${:.2} ({:+.1}%)",
            sol_rate,
            (volatility.sol_trend - 1.0) * 100.0
        );
        println!(
            "  APPLE: ${:.2} ({:+.1}%)",
            apple_rate,
            (volatility.apple_trend - 1.0) * 100.0
        );
        println!();
    }

    Ok(())
}

/// Calculate the total portfolio value for a user in USDT
fn calculate_portfolio_value(
    user: &User,
    exchange_rate_provider: &mut std::sync::RwLockWriteGuard<
        crate::exchange_rate_provider::ExchangeRateProvider,
    >,
) -> f64 {
    let mut total_value = 0.0;

    for (asset, balance) in &user.balances {
        match asset {
            Asset::USDT => total_value += balance,
            _ => {
                if let Some(rate) = exchange_rate_provider.get_rate(asset, &Asset::USDT) {
                    total_value += balance * rate;
                }
            }
        }
    }

    total_value
}

/// Generate comprehensive PnL report for all traders
fn generate_pnl_report(bots: &[TraderBot], exchange: &Exchange) {
    println!("\n📊 COMPREHENSIVE PROFIT & LOSS REPORT");
    println!("========================================");

    let mut rate_provider = get_rate_provider();

    for bot in bots {
        if let Some(user) = exchange.users.get(&bot.address) {
            let current_value = calculate_portfolio_value(user, &mut rate_provider);
            let total_pnl = current_value - bot.pnl_tracker.initial_portfolio_value;
            let pnl_percentage = if bot.pnl_tracker.initial_portfolio_value > 0.0 {
                (total_pnl / bot.pnl_tracker.initial_portfolio_value) * 100.0
            } else {
                0.0
            };

            println!(
                "\nTrader: {} ({})",
                bot.pnl_tracker.trader_name, bot.strategy
            );
            println!(
                "   Initial Portfolio Value: ${:.2}",
                bot.pnl_tracker.initial_portfolio_value
            );
            println!("   💰 Current Portfolio Value: ${:.2}", current_value);
            println!(
                "   📊 Total PnL: ${:.2} ({:+.2}%)",
                total_pnl, pnl_percentage
            );

            // Asset breakdown
            println!("   Asset Holdings:");
            for (asset, balance) in &user.balances {
                if *balance > 0.0 {
                    let asset_value = match asset {
                        Asset::USDT => *balance,
                        _ => {
                            if let Some(rate) = rate_provider.get_rate(asset, &Asset::USDT) {
                                balance * rate
                            } else {
                                0.0
                            }
                        }
                    };
                    println!("     {} {}: ${:.2}", asset, balance, asset_value);
                }
            }

            if bot.last_exercise_round.is_some() {
                println!("   [Note] Recently exercised options and engaged in post-exercise trading");
            }
        }
    }

    // Summary statistics
    let mut total_initial = 0.0;
    let mut total_current = 0.0;

    for bot in bots {
        total_initial += bot.pnl_tracker.initial_portfolio_value;
        if let Some(user) = exchange.users.get(&bot.address) {
            total_current += calculate_portfolio_value(user, &mut rate_provider);
        }
    }

    let market_pnl = total_current - total_initial;
    let market_pnl_pct = if total_initial > 0.0 {
        (market_pnl / total_initial) * 100.0
    } else {
        0.0
    };

    println!("\nMARKET SUMMARY:");
    println!("   Total Initial Value: ${:.2}", total_initial);
    println!("   Total Current Value: ${:.2}", total_current);
    println!(
        "   Market PnL: ${:.2} ({:+.2}%)",
        market_pnl, market_pnl_pct
    );
    println!("========================================");
}
