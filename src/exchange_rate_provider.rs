use crate::Address;
use crate::asset::Asset;
use crate::rbac::{NamedRole, RoleAuthorizer};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
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

pub fn default_exchange_rate_provider_admin_address() -> Address {
    Address::from("0xb73B0A92544a5D2523F00F868d795d50DbDfcCf4")
        .expect("Invalid exchange rate provider admin address literal")
}

// should be used as a singleton
pub struct ExchangeRateProvider {
    exchange_rates: HashMap<AssetPair, f64>, // map pair to quote_amount
    authorizer: RoleAuthorizer,
}

impl ExchangeRateProvider {
    pub fn new() -> ExchangeRateProvider {
        let role_manager_addr = default_exchange_rate_provider_admin_address();

        let mut provider = ExchangeRateProvider {
            exchange_rates: HashMap::new(),
            authorizer: RoleAuthorizer::new(role_manager_addr.clone()),
        };

        // Set admin for module
        let admin_addr = default_exchange_rate_provider_admin_address();
        let admin_role = NamedRole("Admin".to_string());
        provider
            .authorizer
            .make_role_known(admin_role.clone(), role_manager_addr.clone())
            .expect("Panic: role manager of exchange rate provider should be able to add admin role, unless sth's wrong with the setup");

        provider
            .authorizer
            .assign_role(admin_role.clone(), admin_addr, role_manager_addr.clone())
            .expect("Panic: role manager of exchange rate provider should be able to assign admin role, unless sth's wrong with the setup");

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
            quote: Asset::USDT,
        };
        provider.exchange_rates.insert(btc_usdt_pair, 100_000.0);

        return provider;
    }

    pub fn get_rate(&self, base: &Asset, quote: &Asset) -> Option<f64> {
        let pair = AssetPair::from(base.clone(), quote.clone());
        self.exchange_rates.get(&pair).copied()
    }

    pub fn set_rate(
        &mut self,
        base: Asset,
        quote: Asset,
        rate: f64,
        caller_address: Address,
    ) -> Result<(), String> {
        let admin_role = NamedRole("Admin".to_string());
        match self
            .authorizer
            .only_authorized_role(&[admin_role], caller_address)
        {
            Ok(()) => {}
            Err(e) => {
                return Err("caller not authorized to update exchange rate".into());
            }
        }

        let pair: AssetPair = AssetPair::from(base, quote);
        self.exchange_rates.insert(pair, rate);

        Ok(())
    }
}

// Replace OnceCell<ExchangeRateProvider> with OnceCell<RwLock<ExchangeRateProvider>>
static EXCHANGE_RATE_PROVIDER: OnceCell<RwLock<ExchangeRateProvider>> = OnceCell::new();

// Return a read guard. Propagate poisoning via the returned Result.
pub fn get_readonly_rate_provider() -> RwLockReadGuard<'static, ExchangeRateProvider> {
    let lock: &'static RwLock<ExchangeRateProvider> =
        EXCHANGE_RATE_PROVIDER.get_or_init(|| RwLock::new(ExchangeRateProvider::new()));

    return lock
        .read()
        .expect("Panic: could not get or init exchange rate provider");
}

// Return a write guard. Propagate poisoning via the returned Result so callers can handle it.
pub fn get_rate_provider() -> RwLockWriteGuard<'static, ExchangeRateProvider> {
    let lock: &'static RwLock<ExchangeRateProvider> =
        EXCHANGE_RATE_PROVIDER.get_or_init(|| RwLock::new(ExchangeRateProvider::new()));
    return lock
        .write()
        .expect("Panic: could not get or init exchange rate provider");
}
