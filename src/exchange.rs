use crate::address::Address;
use crate::exchange_rate_provider::get_exchange_rate_provider;
use crate::listing_option::ListingOption;
use crate::rbac::RoleAuthorizer;
use crate::types::ListingType;
use crate::user::User;
use crate::utils::are_addresses_equal;
use std::collections::HashMap;

pub fn default_escrow_address() -> Address {
    Address::from("0x0000000000000000000000000000000000000000")
        .expect("Invalid default escrow address literal")
}

pub fn default_exchange_admin_address() -> Address {
    Address::from("0xb73B0A92544a5D2523F00F868d795d50DbDfcCf4")
        .expect("Invalid exchange admin address literal")
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
    pub role_authorizer: RoleAuthorizer,
}

impl Exchange {
    pub fn new() -> Exchange {
        // Init exchange rate provider if it hasn't been initialized

        Exchange {
            users: HashMap::new(),
            escrow_user: User::new(default_escrow_address()),
            listings: HashMap::new(),
            next_listing_id: 1,
            beneficiary_fee_bps: 10, // default to 0.1%
            grantor_fee_bps: 10,     // default to 0.1%
            market_admin_address: Address::from("0x953674f672475ec0A1aBE55156400c6F0086E90a")
                .expect("failed to initialize market admin address"),

            // Init RBAC authorizer (TODO: refactor to make a dedicated service handle auth in v2)
            role_authorizer: RoleAuthorizer::new(),
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
        caller_address: Address,
        option: ListingOption,
    ) -> Result<u32, String> {
        // Get a mutable reference to the seller (we already checked it exists)
        let seller = self.get_user_or_error(&caller_address)?;

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
        caller_address: Address,
    ) -> Result<(), String> {
        let listing_immut = self
            .listings
            .get(&listing_id)
            .ok_or_else(|| String::from("Listing not found"))?;

        if !are_addresses_equal(&caller_address, &listing_immut.grantor_address) {
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
            let grantor = self.get_user_or_error(&caller_address)?;
            grantor.add_asset(&option.base_asset, collateral_amount);
        } else {
            // For PUT options, collateral is in quote_asset (e.g., USDT)
            let collateral_price = option.get_collateral_price(); // strike_price * 100
            self.escrow_user
                .deduct_asset(&option.quote_asset, collateral_price)?;
            let grantor = self.get_user_or_error(&caller_address)?;
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

            // Exhaustive state transition check
            match (option.is_purchased, option.is_unlisted, option.is_exercised) {
                // Valid case first
                (false, false, false) => {}
                (true, _, _) => return Err("Option already purchased!".into()),
                (_, true, _) => return Err("Option has been unlisted!".into()),
                (_, _, true) => return Err("Option has already been exercised!".into()),
            }

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
        option.is_purchased = true;

        Ok(())
    }

    pub fn exercise_option(
        &mut self,
        listing_id: u32,
        caller_address: Address,
    ) -> Result<(), String> {
        // Immutable borrow
        let (exercised_amount, exercised_asset) = {
            let option_immut = self.get_listing_or_error_immutable(listing_id)?;

            let is_beneficiary = are_addresses_equal(
                &caller_address,
                option_immut
                    .beneficiary_address
                    .as_ref()
                    .expect("panic: listed option doesn't have beneficiary address"),
            );

            // Exhaustive state validity check
            match (
                option_immut.is_purchased,
                option_immut.is_unlisted,
                option_immut.is_exercised,
                is_beneficiary,
            ) {
                // Valid case first
                (true, false, false, true) => {}
                (false, false, false, true) => {
                    panic!("panic: option has beneficiary but isn't purchased!")
                }
                (_, _, _, false) => return Err("Caller is not beneficiary of option!".into()),
                (_, true, _, _) => return Err("Option has been unlisted!".into()),
                (_, _, true, _) => return Err("Option has already been exercised!".into()),
            }

            (
                option_immut.get_collateral_price(),
                option_immut.get_exercised_asset().clone(),
            )
        };

        // Perform transfers
        let option = self.get_listing_or_error(listing_id)?;
        option.is_exercised = true;

        {
            assert!(self.escrow_user.get_balance(&exercised_asset) > exercised_amount);
            let beneficiary = self.get_user_or_error(&caller_address)?;
            beneficiary.add_asset(&exercised_asset, exercised_amount);
            self.escrow_user
                .deduct_asset(&exercised_asset, exercised_amount)?;
        }

        Ok(())
    }
    // TODO: allow re-selling of acquired options contract
}
