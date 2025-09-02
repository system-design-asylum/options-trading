use options_trading::{Exchange, User, ListingOption, Asset, ListingType, Address};
use chrono::{Utc, Duration};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_address(suffix: &str) -> Address {
        let base = "0x123456789012345678901234567890123456789";
        let full_address = format!("{}{}", base, suffix);
        Address::from(&full_address).unwrap()
    }

    fn setup_market_and_users() -> (Exchange, Address, Address) {
        let mut market = Exchange::new();
        let seller_addr = create_test_address("1");
        let buyer_addr = create_test_address("2");

        let mut seller = User::new(seller_addr.clone());
    seller.add_asset(&Asset::BTC, 10.0).unwrap();
    seller.add_asset(&Asset::USDT, 1000000.0).unwrap();

    let mut buyer = User::new(buyer_addr.clone());
    buyer.add_asset(&Asset::USDT, 200000.0).unwrap();

        market.users.insert(seller_addr.clone(), seller);
        market.users.insert(buyer_addr.clone(), buyer);

        (market, seller_addr, buyer_addr)
    }

    #[test]
    fn test_exercise_call_success() {
        let (mut market, seller_addr, buyer_addr) = setup_market_and_users();

        // Seller lists CALL option
        let option = ListingOption {
            listing_id: 0,
            base_asset: Asset::BTC,
            quote_asset: Asset::USDT,
            listing_type: ListingType::CALL,
            strike_price: 50000.0,
            ask_price: 500.0,
            bid_price: 490.0,
            expiration_time: Utc::now() + Duration::days(30),
            exercise_amount: 1.0,
            is_purchased: false,
            is_unlisted: false,
            is_exercised: false,
            grantor_address: seller_addr.clone(),
            beneficiary_address: None,
        };

        let listing_id = market.list_option(seller_addr.clone(), option).unwrap();
        // Buyer purchases
        market.purchase_option(listing_id, buyer_addr.clone()).unwrap();

        // Beneficiary (buyer) exercises
        market.exercise_option(listing_id, buyer_addr.clone()).unwrap();

        // After exercise: buyer receives 1 BTC from escrow, pays strike (1 * 50000 USDT)
        let buyer = market.users.get(&buyer_addr).unwrap();
        assert_eq!(buyer.get_balance(&Asset::BTC), 1.0);

        let seller = market.users.get(&seller_addr).unwrap();
        // Seller should have received strike price in USDT (minus fees taken earlier in purchase)
        assert!(seller.get_balance(&Asset::USDT) > 1000000.0);

        // Listing marked exercised
        let listing = market.listings.get(&listing_id).unwrap();
        assert!(listing.is_exercised);
    }

    #[test]
    fn test_exercise_non_beneficiary_fails() {
        let (mut market, seller_addr, buyer_addr) = setup_market_and_users();
        let other_addr = create_test_address("3");

        let option = ListingOption {
            listing_id: 0,
            base_asset: Asset::BTC,
            quote_asset: Asset::USDT,
            listing_type: ListingType::CALL,
            strike_price: 50000.0,
            ask_price: 500.0,
            bid_price: 490.0,
            expiration_time: Utc::now() + Duration::days(30),
            exercise_amount: 1.0,
            is_purchased: false,
            is_unlisted: false,
            is_exercised: false,
            grantor_address: seller_addr.clone(),
            beneficiary_address: None,
        };

        let listing_id = market.list_option(seller_addr.clone(), option).unwrap();
        market.purchase_option(listing_id, buyer_addr.clone()).unwrap();

        let result = market.exercise_option(listing_id, other_addr);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Caller is not beneficiary of option!");
    }

    #[test]
    fn test_exercise_already_exercised_fails() {
        let (mut market, seller_addr, buyer_addr) = setup_market_and_users();

        let option = ListingOption {
            listing_id: 0,
            base_asset: Asset::BTC,
            quote_asset: Asset::USDT,
            listing_type: ListingType::CALL,
            strike_price: 50000.0,
            ask_price: 500.0,
            bid_price: 490.0,
            expiration_time: Utc::now() + Duration::days(30),
            exercise_amount: 1.0,
            is_purchased: false,
            is_unlisted: false,
            is_exercised: false,
            grantor_address: seller_addr.clone(),
            beneficiary_address: None,
        };

        let listing_id = market.list_option(seller_addr.clone(), option).unwrap();
        market.purchase_option(listing_id, buyer_addr.clone()).unwrap();
        market.exercise_option(listing_id, buyer_addr.clone()).unwrap();

        // Attempt to exercise again
        let result = market.exercise_option(listing_id, buyer_addr.clone());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Option has already been exercised!");
    }

    #[test]
    fn test_exercise_put_success() {
        // Setup a market where seller (grantor) escrows quote asset and buyer has base asset to exercise
        let mut market = Exchange::new();
        let seller_addr = create_test_address("4");
        let buyer_addr = create_test_address("5");

        let mut seller = User::new(seller_addr.clone());
    seller.add_asset(&Asset::USDT, 10000.0).unwrap(); // seller will need quote collateral

    let mut buyer = User::new(buyer_addr.clone());
    buyer.add_asset(&Asset::USDT, 100000.0).unwrap(); // to purchase premium
    buyer.add_asset(&Asset::ETH, 5.0).unwrap(); // buyer must have base asset to transfer on exercise

        market.users.insert(seller_addr.clone(), seller);
        market.users.insert(buyer_addr.clone(), buyer);

        // Seller lists a PUT option for 1 ETH at strike 3000 USDT
        let option = ListingOption {
            listing_id: 0,
            base_asset: Asset::ETH,
            quote_asset: Asset::USDT,
            listing_type: ListingType::PUT,
            strike_price: 3000.0,
            ask_price: 20.0,
            bid_price: 18.0,
            expiration_time: Utc::now() + Duration::days(30),
            exercise_amount: 1.0,
            is_purchased: false,
            is_unlisted: false,
            is_exercised: false,
            grantor_address: seller_addr.clone(),
            beneficiary_address: None,
        };

        let listing_id = market.list_option(seller_addr.clone(), option).unwrap();

        // Buyer purchases
        market.purchase_option(listing_id, buyer_addr.clone()).unwrap();

        // Beneficiary exercises: buyer should send 1 ETH to seller and receive 3000 USDT from escrow
        market.exercise_option(listing_id, buyer_addr.clone()).unwrap();

        let buyer = market.users.get(&buyer_addr).unwrap();
        assert_eq!(buyer.get_balance(&Asset::ETH), 4.0); // 5 - 1 ETH
        assert!(buyer.get_balance(&Asset::USDT) > 100000.0); // Received strike in USDT (minus fees earlier)

        let seller = market.users.get(&seller_addr).unwrap();
        assert_eq!(seller.get_balance(&Asset::ETH), 1.0); // seller received 1 ETH from buyer

        let listing = market.listings.get(&listing_id).unwrap();
        assert!(listing.is_exercised);
    }
}
