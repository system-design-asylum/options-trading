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

    #[test]
    fn test_exercise_call_insufficient_quote_fails() {
        let mut market = Exchange::new();
        let seller = create_test_address("1");
        let buyer = create_test_address("2");

        let mut s = User::new(seller.clone());
        s.add_asset(&Asset::BTC, 2.0).unwrap();
        s.add_asset(&Asset::USDT, 0.0).unwrap();

        let mut b = User::new(buyer.clone());
        // Buyer has just enough to purchase but not enough to exercise
        b.add_asset(&Asset::USDT, 100000.0).unwrap();
        market.users.insert(seller.clone(), s);
        market.users.insert(buyer.clone(), b);

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
            grantor_address: seller.clone(),
            beneficiary_address: None,
        };

        let lid = market.list_option(seller.clone(), option).unwrap();
        market.purchase_option(lid, buyer.clone()).unwrap();

        // After purchase buyer will have ~49,950 USDT, which is less than strike 50,000 => exercise fails
        let res = market.exercise_option(lid, buyer.clone());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Insufficient USDT balance");
    }

    #[test]
    fn test_exercise_put_insufficient_base_fails() {
        let mut market = Exchange::new();
        let seller = create_test_address("3");
        let buyer = create_test_address("4");

        let mut s = User::new(seller.clone());
        // Seller must escrow quote (USDT)
        s.add_asset(&Asset::USDT, 5000.0).unwrap();

        let mut b = User::new(buyer.clone());
        // Buyer has no ETH to deliver when exercising
        b.add_asset(&Asset::USDT, 100000.0).unwrap();
        b.add_asset(&Asset::ETH, 0.0).unwrap();

        market.users.insert(seller.clone(), s);
        market.users.insert(buyer.clone(), b);

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
            grantor_address: seller.clone(),
            beneficiary_address: None,
        };

        let lid = market.list_option(seller.clone(), option).unwrap();
        market.purchase_option(lid, buyer.clone()).unwrap();

        let res = market.exercise_option(lid, buyer.clone());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Insufficient ETH balance");
    }

    #[test]
    fn test_exercise_after_expiration_fails() {
        let mut market = Exchange::new();
        let seller = create_test_address("5");
        let buyer = create_test_address("6");

        let mut s = User::new(seller.clone());
        s.add_asset(&Asset::BTC, 2.0).unwrap();
        s.add_asset(&Asset::USDT, 0.0).unwrap();

        let mut b = User::new(buyer.clone());
        b.add_asset(&Asset::USDT, 200000.0).unwrap();

        market.users.insert(seller.clone(), s);
        market.users.insert(buyer.clone(), b);

        // Create option already expired
        let option = ListingOption {
            listing_id: 0,
            base_asset: Asset::BTC,
            quote_asset: Asset::USDT,
            listing_type: ListingType::CALL,
            strike_price: 50000.0,
            ask_price: 500.0,
            bid_price: 490.0,
            expiration_time: Utc::now() - Duration::days(1),
            exercise_amount: 1.0,
            is_purchased: false,
            is_unlisted: false,
            is_exercised: false,
            grantor_address: seller.clone(),
            beneficiary_address: None,
        };

        let lid = market.list_option(seller.clone(), option).unwrap();
        // Purchase still allowed; exercise should fail due to expiration
        market.purchase_option(lid, buyer.clone()).unwrap();
        let res = market.exercise_option(lid, buyer.clone());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Option has expired!");
    }

    #[test]
    fn test_exercise_by_seller_fails() {
        let mut market = Exchange::new();
        let seller = create_test_address("7");
        let buyer = create_test_address("8");

        let mut s = User::new(seller.clone());
        s.add_asset(&Asset::BTC, 2.0).unwrap();
        s.add_asset(&Asset::USDT, 0.0).unwrap();

        let mut b = User::new(buyer.clone());
        b.add_asset(&Asset::USDT, 200000.0).unwrap();

        market.users.insert(seller.clone(), s);
        market.users.insert(buyer.clone(), b);

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
            grantor_address: seller.clone(),
            beneficiary_address: None,
        };

        let lid = market.list_option(seller.clone(), option).unwrap();
        market.purchase_option(lid, buyer.clone()).unwrap();

        // Seller attempting to exercise should fail
        let res = market.exercise_option(lid, seller.clone());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Caller is not beneficiary of option!");
    }
}
