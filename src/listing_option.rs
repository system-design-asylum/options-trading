use crate::Address;
use crate::asset::Asset;
use crate::types::ListingType;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ListingOption {
    pub listing_id: u32,
    pub base_asset: Asset,
    pub quote_asset: Asset,
    pub listing_type: ListingType,
    pub strike_price: f64, // based on base asset
    pub ask_price: f64,    // based on quote asset
    pub bid_price: f64,    // based on quote asset
    pub expiration_time: DateTime<Utc>,
    pub grantor_address: Address,
    pub beneficiary_address: Option<Address>, // the one who has the right to buy/sell, defaults to None

    // State transitions
    pub is_purchased: bool,
    pub is_unlisted: bool,
    pub is_exercised: bool, //  whether the option contract has been exercised by the beneficiary or not
}

impl ListingOption {
    // TODO: add new() function here

    pub fn get_premium_price(&self) -> f64 {
        self.ask_price * 100.0
    }

    /// Return the collateral amount that the grantor has to pay upfront upon listing
    pub fn get_collateral_price(&self) -> f64 {
        self.strike_price * 100.0
    }

    pub fn get_exercised_asset(&self) -> &Asset {
        match self.listing_type {
            ListingType::CALL => &self.base_asset,
            ListingType::PUT => &self.quote_asset,
        }
    }
}
