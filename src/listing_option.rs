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
    pub exercise_amount: f64,                 // based on quote asset

    // State transitions
    pub is_purchased: bool,
    pub is_unlisted: bool,
    pub is_exercised: bool, //  whether the option contract has been exercised by the beneficiary or not
}

impl ListingOption {
    // TODO: add new() function here
    pub fn new(
        listing_id: u32,
        base_asset: Asset,
        quote_asset: Asset,
        listing_type: ListingType,
        strike_price: f64,
        ask_price: f64,
        bid_price: f64,
        expiration_time: DateTime<Utc>,
        grantor_address: Address,
        beneficiary_address: Option<Address>,
        exercise_amount: f64,
        is_purchased: bool,
        is_unlisted: bool,
        is_exercised: bool,
    ) -> Self {
        ListingOption {
            listing_id,
            base_asset,
            quote_asset,
            listing_type,
            strike_price,
            ask_price,
            bid_price,
            expiration_time,
            grantor_address,
            beneficiary_address,
            exercise_amount,
            is_purchased,
            is_unlisted,
            is_exercised,
        }
    }

    pub fn get_premium_price(&self) -> f64 {
        self.ask_price * 100.0
    }

    pub fn get_buy_amount(&self, is_for_grantor: bool) -> f64 {
        match (self.listing_type.clone(), is_for_grantor) {
            (ListingType::CALL, true) => self.exercise_amount * self.strike_price,
            (ListingType::PUT, true) => self.exercise_amount,

            (ListingType::CALL, false) => self.exercise_amount,
            (ListingType::PUT, false) => self.exercise_amount * self.strike_price,
        }
    }

    pub fn get_sell_amount(&self, is_for_grantor: bool) -> f64 {
        match (self.listing_type.clone(), is_for_grantor) {
            (ListingType::CALL, true) => self.exercise_amount,
            (ListingType::PUT, true) => self.exercise_amount * self.strike_price,

            (ListingType::CALL, false) => self.exercise_amount * self.strike_price,
            (ListingType::PUT, false) => self.exercise_amount,
        }
    }

    pub fn get_buy_asset(&self, is_for_grantor: bool) -> &Asset {
        match (self.listing_type.clone(), is_for_grantor) {
            (ListingType::CALL, true) => &self.quote_asset,
            (ListingType::PUT, true) => &self.base_asset,

            (ListingType::CALL, false) => &self.base_asset,
            (ListingType::PUT, false) => &self.quote_asset,
        }
    }

    pub fn get_sell_asset(&self, is_for_grantor: bool) -> &Asset {
        match (self.listing_type.clone(), is_for_grantor) {
            (ListingType::CALL, true) => &self.base_asset,
            (ListingType::PUT, true) => &self.quote_asset,

            (ListingType::CALL, false) => &self.quote_asset,
            (ListingType::PUT, false) => &self.base_asset,
        }
    }
}
