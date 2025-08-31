use crate::exchange_rate_provider::{get_exchange_rate_provider};
use crate::listing_option::ListingOption;
use crate::address::Address;
use crate::types::{ListingType};
use crate::user::User;
use crate::utils::are_addresses_equal;
use std::collections::HashMap;

pub fn escrow_address() -> Address {
    Address::from("0x0000000000000000000000000000000000000000")
        .expect("Invalid escrow address literal")
}

const MAX_FEE_BPS: u16 = 10_000;

pub struct Exchange {
    pub users: HashMap<Address, User>,
    pub escrow_user: User,
    pub listings: HashMap<u32, ListingOption>,
    pub next_listing_id: u32,

    pub beneficiary_fee_bps: u16, // basis points (10_000 for 100%)
    pub grantor_fee_bps: u16,

    pub market_admin_address: Address,
}

impl Exchange {
    pub fn new() -> Exchange {
        // init exchange rate provider if it hasn't been initialized

        Exchange {
            users: HashMap::new(),
            escrow_user: User::new(escrow_address()),
            listings: HashMap::new(),
            next_listing_id: 1,
            beneficiary_fee_bps: 10, // default to 0.1%
            grantor_fee_bps: 10,     // default to 0.1%
            market_admin_address: Address::from("0x953674f672475ec0A1aBE55156400c6F0086E90a")
                .expect("failed to initialize market admin address"),
        }
    }

    /** Getter funcs */
    pub fn get_user_or_error(&mut self, user_address: &Address) -> Result<&mut User, String> {
        self.users
            .get_mut(&user_address)
            .ok_or_else(|| String::from("User not found"))
    }

    pub fn get_user_or_error_immutable(&self, user_address: &Address) -> Result<&User, String> {
        self.users
            .get(&user_address)
            .ok_or_else(|| String::from("User not found"))
    }

    pub fn get_listing_or_error(&mut self, listing_id: u32) -> Result<&mut ListingOption, String> {
        self.listings
            .get_mut(&listing_id)
            .ok_or_else(|| String::from("Listing not found"))
    }

    pub fn get_listing_or_error_immutable(
        &self,
        listing_id: u32,
    ) -> Result<&ListingOption, String> {
        self.listings
            .get(&listing_id)
            .ok_or_else(|| String::from("Listing not found"))
    }

    pub fn get_beneficiary_fee(&self, premium_price: f64) -> f64 {
        premium_price * (self.beneficiary_fee_bps as f64) / (MAX_FEE_BPS as f64)
    }

    pub fn get_grantor_fee(&self, premium_price: f64) -> f64 {
        premium_price * (self.grantor_fee_bps as f64) / (MAX_FEE_BPS as f64)
    }
    /** */

    /** Setters */
    pub fn set_beneficiary_fee_bps(
        &mut self,
        new_bps: u16,
        caller_address: Address,
    ) -> Result<(), String> {
        if !are_addresses_equal(&caller_address, &self.market_admin_address) {
            return Err("Only market admin only".into());
        }
        if new_bps > 10_000 {
            return Err("Invalid bps, must be between 0 - 10.000".into());
        }
        self.beneficiary_fee_bps = new_bps;

        Ok(())
    }

    pub fn set_grantor_fee_bps(
        &mut self,
        new_bps: u16,
        caller_address: Address,
    ) -> Result<(), String> {
        if !are_addresses_equal(&caller_address, &self.market_admin_address) {
            return Err("Only market admin only".into());
        }
        if new_bps > 10_000 {
            return Err("Invalid bps, must be between 0 - 10.000".into());
        }
        self.grantor_fee_bps = new_bps;

        Ok(())
    }

    pub fn list_option(
        &mut self,
        grantor_address: Address,
        option: ListingOption,
    ) -> Result<u32, String> {
        // Get a mutable reference to the seller (we already checked it exists)
        let seller = self.get_user_or_error(&grantor_address)?;

        // sanity check
        if option.listing_type == ListingType::CALL {
            // For CALL options, seller must deposit base asset (e.g., BTC) as collateral
            let base_balance = seller.get_balance(&option.base_asset);
            // For simplicity, 1 contract = 1 unit of base asset (not 100 like traditional options)
            let collateral_amount = 1.0; // 1 unit of base asset per contract
            if base_balance < collateral_amount {
                return Err("Insufficient base asset balance to cover CALL option".into());
            }
            seller.deduct_asset(&option.base_asset, collateral_amount)?;
            self.escrow_user
                .add_asset(&option.base_asset, collateral_amount);
        } else {
            // For PUT options, seller must deposit quote asset (e.g., USDT) as collateral
            let quote_balance = seller.get_balance(&option.quote_asset);
            let collateral_price = option.get_collateral_price(); // strike_price * 100
            if quote_balance < collateral_price {
                return Err("Insufficient quote asset balance to cover PUT option".into());
            }
            seller.deduct_asset(&option.quote_asset, collateral_price)?;
            self.escrow_user
                .add_asset(&option.quote_asset, collateral_price);
        }

        // store into listings
        let listing_id = self.next_listing_id;
        self.next_listing_id += 1;
        let mut option_with_id = option;
        option_with_id.listing_id = listing_id;
        self.listings.insert(listing_id, option_with_id);

        Ok(listing_id)
    }

    /// Unlist a previously listed option
    pub fn unlist_option(
        &mut self,
        listing_id: u32,
        grantor_address: Address,
    ) -> Result<(), String> {
        let listing_immut = self
            .listings
            .get(&listing_id)
            .ok_or_else(|| String::from("Listing not found"))?;

        if !are_addresses_equal(&grantor_address, &listing_immut.grantor_address) {
            return Err("Only the seller can unlist this option".into());
        }

        match listing_immut.beneficiary_address.as_ref() {
            Some(address) => {
                println!("Beneficiary address exists: {:?}, cannot unlist", address);
                return Err("Option has been acquired, cannot unlist".into());
            }
            None => println!("Option is not acquired, can unlist"),
        }
        // immutable borrow of listing ends here

        // Remove the listing to take ownership (no longer borrowing self.listings)
        let option = self
            .listings
            .remove(&listing_id)
            .ok_or_else(|| String::from("Listing not found"))?;

        // Return collateral to grantor
        if option.listing_type == ListingType::CALL {
            // For CALL options, collateral is in base_asset (e.g., BTC)
            let collateral_amount = 1.0; // Same as what we deposited in list_option
            self.escrow_user
                .deduct_asset(&option.base_asset, collateral_amount)?;
            let grantor = self.get_user_or_error(&grantor_address)?;
            grantor.add_asset(&option.base_asset, collateral_amount);
        } else {
            // For PUT options, collateral is in quote_asset (e.g., USDT)
            let collateral_price = option.get_collateral_price(); // strike_price * 100
            self.escrow_user
                .deduct_asset(&option.quote_asset, collateral_price)?;
            let grantor = self.get_user_or_error(&grantor_address)?;
            grantor.add_asset(&option.quote_asset, collateral_price);
        }

        Ok(())
    }

    pub fn purchase_option(
        &mut self,
        listing_id: u32,
        beneficiary_address: Address,
    ) -> Result<(), String> {
        // Borrow listing immutably to compute prices and fees.
        let (premium_price, quote_asset, grantor_address, beneficiary_fee, grantor_fee) = {
            let option = self.get_listing_or_error_immutable(listing_id)?;
            let premium_price = option.get_premium_price();
            let beneficiary_fee = self.get_beneficiary_fee(premium_price);
            let grantor_fee = self.get_grantor_fee(premium_price);
            // clone fields needed later while we still have the immutable borrow
            (
                premium_price,
                option.quote_asset.clone(),
                option.grantor_address.clone(),
                beneficiary_fee,
                grantor_fee,
            )
        };

        // Compute amounts
        let amt_from_beneficiary = premium_price + beneficiary_fee;
        let amt_to_grantor = premium_price - grantor_fee;

        // Deduct from beneficiary and collect fee
        {
            let beneficiary = self.get_user_or_error(&beneficiary_address)?;
            if beneficiary.get_balance(&quote_asset) < amt_from_beneficiary {
                return Err("Buyer doesn't have enough quote balance to purchase option".into());
            }
            beneficiary.deduct_asset(&quote_asset, amt_from_beneficiary)?;
        }
        self.escrow_user.add_asset(&quote_asset, beneficiary_fee);

        // Dispatch money to grantor and collect fee
        {
            let grantor = self.get_user_or_error(&grantor_address)?;
            grantor.add_asset(&quote_asset, amt_to_grantor);
        }
        self.escrow_user.add_asset(&quote_asset, grantor_fee);

        // Mutate the listing (no other borrows active)
        let option = self.get_listing_or_error(listing_id)?;
        option.beneficiary_address = Some(beneficiary_address);

        Ok(())
    }
}
