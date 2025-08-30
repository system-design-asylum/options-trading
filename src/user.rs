// user.rs - User management and account operations

use strum::IntoEnumIterator;

use crate::types::{Address, Asset};
use std::collections::HashMap;

/// Represents a user in the trading system
#[derive(Clone)]
pub struct User {
    pub address: Address,
    pub balances: HashMap<Asset, f64>, // map from asset to asset balance
}

impl User {
    // /// Create a new user with initial cash and asset holdings
    pub fn new(address: Address) -> Self {
        let mut balances = HashMap::new();

        // Give users some initial assets
        // balances.insert(Asset::BTC, 10.0);
        // balances.insert(Asset::ETH, 50.0);
        // balances.insert(Asset::SOL, 100.0);
        // balances.insert(Asset::APPLE, 20.0);

        for asset in Asset::iter() {
            balances.insert(asset, 0.0);
        }

        User { address, balances }
    }

    /// Get the balance of a specific asset
    pub fn get_balance(&self, asset: &Asset) -> f64 {
        *self.balances.get(asset).unwrap_or(&0.0)
    }

    /// Add assets to the user's portfolio
    pub fn add_asset(&mut self, asset: &Asset, amount: f64) {
        let balance = self.balances.entry(asset.clone()).or_insert(0.0);
        *balance += amount;
    }

    /// Deduct assets from the user's portfoli
    pub fn deduct_asset(&mut self, asset: &Asset, amount: f64) -> Result<(), String> {
        let balance = self.balances.entry(asset.clone()).or_insert(0.0);
        if *balance < amount {
            return Err(format!("Insufficient {} balance", asset));
        }
        *balance -= amount;
        Ok(())
    }
}
