use chrono::{Duration, Utc};
use options_trading::{Address, Asset, Exchange, ListingOption, ListingType, User};

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_test_address(suffix: &str) -> Address {
        let base = "0x123456789012345678901234567890123456789";
        let full_address = format!("{}{}", base, suffix);
        Address::from(&full_address).unwrap()
    }

    #[test]
    fn test_complete_options_trading_workflow() {
        // Setup market
        let mut market = Exchange::new();

        // Create users
        let alice_addr = create_test_address("1");
        let bob_addr = create_test_address("2");
        let charlie_addr = create_test_address("3");

        let mut alice = User::new(alice_addr.clone());
        alice.add_asset(&Asset::USDT, 10000000.0); // 10M USDT
        alice.add_asset(&Asset::BTC, 5.0);

        let mut bob = User::new(bob_addr.clone());
        bob.add_asset(&Asset::USDT, 200000.0); // 200k USDT
        bob.add_asset(&Asset::ETH, 100.0);

        let mut charlie = User::new(charlie_addr.clone());
        charlie.add_asset(&Asset::USDT, 400000.0); // Increased to cover PUT option collateral (300k) + buffer
        charlie.add_asset(&Asset::SOL, 500.0);

        market.users.insert(alice_addr.clone(), alice);
        market.users.insert(bob_addr.clone(), bob);
        market.users.insert(charlie_addr.clone(), charlie);

        // Alice lists a BTC CALL option
        let alice_option = ListingOption {
            listing_id: 0,
            base_asset: Asset::BTC,
            quote_asset: Asset::USDT,
            listing_type: ListingType::CALL,
            strike_price: 50000.0,
            ask_price: 500.0,
            bid_price: 490.0,
            expiration_time: Utc::now() + Duration::days(30),
            grantor_address: alice_addr.clone(),
            beneficiary_address: None,
            exercise_amount: 1.0,
            is_purchased: false,
            is_unlisted: false,
            is_exercised: false,
        };

        let alice_listing_id = market
            .list_option(alice_addr.clone(), alice_option)
            .unwrap();
        assert_eq!(alice_listing_id, 1);

        // Check Alice's collateral was deducted (1 BTC for CALL option)
        let alice = market.users.get(&alice_addr).unwrap();
        assert_eq!(alice.get_balance(&Asset::USDT), 10000000.0); // USDT unchanged
        assert_eq!(alice.get_balance(&Asset::BTC), 4.0); // 5 - 1 BTC collateral

        // Bob purchases Alice's option
        let result = market.purchase_option(alice_listing_id, bob_addr.clone());
        assert!(result.is_ok());

        // Verify the purchase
        let listing = market.listings.get(&alice_listing_id).unwrap();
        assert_eq!(listing.beneficiary_address, Some(bob_addr.clone()));

        // Check balances after purchase
        let bob = market.users.get(&bob_addr).unwrap();
        assert_eq!(bob.get_balance(&Asset::USDT), 149950.0); // 200k - 50k premium - 50 fee

        let alice_after = market.users.get(&alice_addr).unwrap();
        assert_eq!(alice_after.get_balance(&Asset::USDT), 10049950.0); // 10M + 50k premium - 50 fee
        assert_eq!(alice_after.get_balance(&Asset::BTC), 4.0); // Still 4 BTC (1 BTC in escrow)

        // Escrow should hold BTC collateral + USDT fees
        assert_eq!(market.escrow_user.get_balance(&Asset::BTC), 1.0); // BTC collateral
        assert_eq!(market.escrow_user.get_balance(&Asset::USDT), 100.0); // 2 fees (50 each)

        // Alice tries to unlist but fails (option was purchased)
        let unlist_result = market.unlist_option(alice_listing_id, alice_addr.clone());
        assert!(unlist_result.is_err());

        // Charlie lists a PUT option
        let charlie_option = ListingOption {
            listing_id: 0,
            base_asset: Asset::ETH,
            quote_asset: Asset::USDT,
            listing_type: ListingType::PUT,
            strike_price: 3000.0,
            ask_price: 200.0,
            bid_price: 190.0,
            expiration_time: Utc::now() + Duration::days(15),
            grantor_address: charlie_addr.clone(),
            beneficiary_address: None,
            exercise_amount: 100.0, // 100 ETH
            is_purchased: false,
            is_unlisted: false,
            is_exercised: false,
        };

        let charlie_listing_id = market
            .list_option(charlie_addr.clone(), charlie_option)
            .unwrap();
        assert_eq!(charlie_listing_id, 2);

        // Charlie unlists his option (before anyone purchases it)
        let unlist_result = market.unlist_option(charlie_listing_id, charlie_addr.clone());
        assert!(unlist_result.is_ok());

        // Verify listing was removed
        assert!(!market.listings.contains_key(&charlie_listing_id));

        // Charlie should get his collateral back
        let charlie_after = market.users.get(&charlie_addr).unwrap();
        assert_eq!(charlie_after.get_balance(&Asset::USDT), 400000.0); // Back to original

        // Final state checks
        assert_eq!(market.listings.len(), 1); // Only Alice's option remains
        assert_eq!(market.next_listing_id, 3); // Next ID should be 3
    }

    #[test]
    fn test_multiple_users_multiple_options() {
        let mut market = Exchange::new();

        // Create 3 users with different assets
        let addrs: Vec<Address> = (1..=3)
            .map(|i| create_test_address(&i.to_string()))
            .collect();

        for (_i, addr) in addrs.iter().enumerate() {
            let mut user = User::new(addr.clone());
            user.add_asset(&Asset::USDT, 1000000.0); // 1M USDT each
            user.add_asset(&Asset::BTC, 1.0);
            user.add_asset(&Asset::ETH, 10.0);
            market.users.insert(addr.clone(), user);
        }

        // Each user lists an option
        let assets = [Asset::BTC, Asset::ETH, Asset::BTC];
        let strikes = [50000.0, 3000.0, 55000.0];
        let premiums = [500.0, 200.0, 600.0];

        let mut listing_ids = Vec::new();

        for (i, addr) in addrs.iter().enumerate() {
            let option = ListingOption {
                listing_id: 0,
                base_asset: assets[i].clone(),
                quote_asset: Asset::USDT,
                listing_type: ListingType::CALL,
                strike_price: strikes[i],
                ask_price: premiums[i],
                bid_price: premiums[i] - 10.0,
                expiration_time: Utc::now() + Duration::days(30),
                grantor_address: addr.clone(),
                beneficiary_address: None,
                exercise_amount: 1.0,
                is_purchased: false,
                is_unlisted: false,
                is_exercised: false,
            };

            let listing_id = market.list_option(addr.clone(), option).unwrap();
            listing_ids.push(listing_id);
        }

        assert_eq!(market.listings.len(), 3);
        assert_eq!(listing_ids, vec![1, 2, 3]);

        // Cross-purchases: user 0 buys from user 1, user 1 buys from user 2, user 2 buys from user 0
        let purchases = [(0, 1), (1, 2), (2, 0)]; // (buyer_index, seller_listing_index)

        for (buyer_idx, listing_idx) in purchases {
            let buyer_addr = &addrs[buyer_idx];
            let listing_id = listing_ids[listing_idx];

            let result = market.purchase_option(listing_id, buyer_addr.clone());
            assert!(result.is_ok());

            // Verify beneficiary was set
            let listing = market.listings.get(&listing_id).unwrap();
            assert_eq!(listing.beneficiary_address, Some(buyer_addr.clone()));
        }

        // All options should now have beneficiaries
        for listing_id in &listing_ids {
            let listing = market.listings.get(listing_id).unwrap();
            assert!(listing.beneficiary_address.is_some());
        }
    }

    #[test]
    fn test_fee_calculations_with_different_rates() {
        let mut market = Exchange::new();
        let admin_addr = market.market_admin_address.clone();

        // Set custom fee rates
        market
            .set_beneficiary_fee_bps(50, admin_addr.clone())
            .unwrap(); // 0.5%
        market.set_grantor_fee_bps(25, admin_addr).unwrap(); // 0.25%

        // Setup users
        let seller_addr = create_test_address("1");
        let buyer_addr = create_test_address("2");

        let mut seller = User::new(seller_addr.clone());
        seller.add_asset(&Asset::USDT, 10000000.0);
        seller.add_asset(&Asset::BTC, 5.0); // Need BTC for CALL option collateral

        let mut buyer = User::new(buyer_addr.clone());
        buyer.add_asset(&Asset::USDT, 200000.0); // Increased to cover premium + fee

        market.users.insert(seller_addr.clone(), seller);
        market.users.insert(buyer_addr.clone(), buyer);

        // List and purchase option
        let option = ListingOption {
            listing_id: 0,
            base_asset: Asset::BTC,
            quote_asset: Asset::USDT,
            listing_type: ListingType::CALL,
            strike_price: 50000.0,
            ask_price: 1000.0, // Higher premium for clearer fee calculation
            bid_price: 990.0,
            expiration_time: Utc::now() + Duration::days(30),
            grantor_address: seller_addr.clone(),
            beneficiary_address: None,
            // Added missing fields:
            exercise_amount: 1.0,
            is_purchased: false,
            is_unlisted: false,
            is_exercised: false,
        };

        let listing_id = market.list_option(seller_addr.clone(), option).unwrap();
        market
            .purchase_option(listing_id, buyer_addr.clone())
            .unwrap();

        // Premium = 1000 * 100 = 100,000
        // Beneficiary fee = 100,000 * 0.005 = 500
        // Grantor fee = 100,000 * 0.0025 = 250

        let buyer = market.users.get(&buyer_addr).unwrap();
        assert_eq!(buyer.get_balance(&Asset::USDT), 99500.0); // 200k - 100k premium - 500 fee

        let seller = market.users.get(&seller_addr).unwrap();
        assert_eq!(seller.get_balance(&Asset::USDT), 10099750.0); // 10M + 100k premium - 250 fee
        assert_eq!(seller.get_balance(&Asset::BTC), 4.0); // 5 - 1 BTC collateral

        // Escrow should have BTC collateral + USDT fees
        assert_eq!(market.escrow_user.get_balance(&Asset::BTC), 1.0); // BTC collateral
        assert_eq!(market.escrow_user.get_balance(&Asset::USDT), 750.0); // 500 + 250 fees
    }
}
