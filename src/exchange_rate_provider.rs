use crate::asset::Asset;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Mutex;
use strum::IntoEnumIterator; // add this so Asset::iter() is in scope // added to allow mutable access to the singleton

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AssetPair {
    base: Asset,
    quote: Asset,
}

impl AssetPair {
    pub fn from(base: Asset, quote: Asset) -> AssetPair {
        AssetPair { base, quote }
    }
}

// should be used as a singleton
struct ExchangeRateProvider {
    exchange_rates: HashMap<AssetPair, f64>, // map pair to quote_amount
	
}

impl ExchangeRateProvider {
    pub fn new() -> ExchangeRateProvider {
        let mut provider = ExchangeRateProvider {
            exchange_rates: HashMap::new(),
        };
        for base_asset in Asset::iter() {
            for quote_asset in Asset::iter() {
                if quote_asset == base_asset {
                    continue;
                }
                let pair = AssetPair::from(base_asset.clone(), quote_asset.clone());
                provider.exchange_rates.insert(pair, 0.0);
            }
        }
		
		// Mock rate setup rates for simulation
		let btc_usdt_pair = AssetPair {
			base: Asset::BTC,
			quote: Asset::USDT
		};
		provider.exchange_rates.insert(btc_usdt_pair, 100_000.0);

        return provider;
    }

    // New getter that returns an Option<f64>
    pub fn get_rate(&self, base: &Asset, quote: &Asset) -> Option<f64> {
        let pair = AssetPair::from(base.clone(), quote.clone());
        self.exchange_rates.get(&pair).copied()
    }

    // New setter to update rates
    pub fn set_rate(&mut self, base: Asset, quote: Asset, rate: f64) {
        let pair = AssetPair::from(base, quote);
        self.exchange_rates.insert(pair, rate);
    }
}

// Replace OnceCell<ExchangeRateProvider> with OnceCell<Mutex<ExchangeRateProvider>>
static EXCHANGE_RATE_PROVIDER: OnceCell<Mutex<ExchangeRateProvider>> = OnceCell::new();

// Return a reference to the Mutex so callers can lock() to mutate/read
pub fn get_exchange_rate_provider() -> &'static Mutex<ExchangeRateProvider> {
    EXCHANGE_RATE_PROVIDER.get_or_init(|| Mutex::new(ExchangeRateProvider::new()))
}
