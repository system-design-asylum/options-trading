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

    fn create_test_option(grantor_address: Address) -> ListingOption {
        ListingOption {
            listing_id: 0, // Will be set by market
            base_asset: Asset::BTC,
            quote_asset: Asset::USDT,
            listing_type: ListingType::CALL,
            strike_price: 50000.0,
            ask_price: 500.0,
            bid_price: 490.0,
            expiration_time: Utc::now() + Duration::days(30),
            grantor_address,
            beneficiary_address: None,
        }
    }

    fn setup_market_with_users() -> (Exchange, Address, Address) {
        let mut market = Exchange::new();
        
        let seller_addr = create_test_address("1");
        let buyer_addr = create_test_address("2");
        
        let mut seller = User::new(seller_addr.clone());
        seller.add_asset(&Asset::USDT, 20000000.0); // 20M USDT for collateral (enough for multiple options)
        seller.add_asset(&Asset::BTC, 10.0);
        
        let mut buyer = User::new(buyer_addr.clone());
        buyer.add_asset(&Asset::USDT, 100000.0); // 100k USDT for purchasing
        buyer.add_asset(&Asset::ETH, 100.0);
        
        market.users.insert(seller_addr.clone(), seller);
        market.users.insert(buyer_addr.clone(), buyer);
        
        (market, seller_addr, buyer_addr)
    }

    #[test]
    fn test_market_creation() {
        let market = Exchange::new();
        
        assert!(market.users.is_empty());
        assert!(market.listings.is_empty());
        assert_eq!(market.next_listing_id, 1);
        assert_eq!(market.beneficiary_fee_bps, 10);
        assert_eq!(market.grantor_fee_bps, 10);
    }

    #[test]
    fn test_get_fees() {
        let market = Exchange::new();
        let premium = 1000.0;
        
        // Default fees are 10 bps = 0.1%
        assert_eq!(market.get_beneficiary_fee(premium), 1.0); // 1000 * 0.001
        assert_eq!(market.get_grantor_fee(premium), 1.0);
    }

    #[test]
    fn test_set_beneficiary_fee_bps_success() {
        let mut market = Exchange::new();
        let admin_addr = market.market_admin_address.clone();
        
        let result = market.set_beneficiary_fee_bps(50, admin_addr);
        assert!(result.is_ok());
        assert_eq!(market.beneficiary_fee_bps, 50);
    }

    #[test]
    fn test_set_beneficiary_fee_bps_unauthorized() {
        let mut market = Exchange::new();
        let unauthorized_addr = create_test_address("1");
        
        let result = market.set_beneficiary_fee_bps(50, unauthorized_addr);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Only market admin only");
        assert_eq!(market.beneficiary_fee_bps, 10); // Should remain unchanged
    }

    #[test]
    fn test_set_beneficiary_fee_bps_invalid() {
        let mut market = Exchange::new();
        let admin_addr = market.market_admin_address.clone();
        
        let result = market.set_beneficiary_fee_bps(10001, admin_addr);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid bps, must be between 0 - 10.000");
        assert_eq!(market.beneficiary_fee_bps, 10); // Should remain unchanged
    }

    #[test]
    fn test_list_option_success() {
        let (mut market, seller_addr, _) = setup_market_with_users();
        let option = create_test_option(seller_addr.clone());
        
        let result = market.list_option(seller_addr.clone(), option);
        assert!(result.is_ok());
        
        let listing_id = result.unwrap();
        assert_eq!(listing_id, 1);
        assert_eq!(market.next_listing_id, 2);
        assert!(market.listings.contains_key(&listing_id));
        
        // Check that collateral was deducted (1 BTC for CALL option)
        let seller = market.users.get(&seller_addr).unwrap();
        assert_eq!(seller.get_balance(&Asset::BTC), 9.0); // 10 - 1 BTC collateral
        
        // Check escrow received collateral
        assert_eq!(market.escrow_user.get_balance(&Asset::BTC), 1.0);
    }

    #[test]
    fn test_list_option_insufficient_collateral() {
        let mut market = Exchange::new();
        let seller_addr = create_test_address("1");
        
        let mut seller = User::new(seller_addr.clone());
        seller.add_asset(&Asset::BTC, 0.5); // Not enough BTC for CALL option collateral
        seller.add_asset(&Asset::USDT, 1000000.0); // Plenty of USDT but not needed for CALL
        market.users.insert(seller_addr.clone(), seller);
        
        let option = create_test_option(seller_addr.clone());
        let result = market.list_option(seller_addr, option);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient base asset balance to cover CALL option");
    }

    #[test]
    fn test_list_option_user_not_found() {
        let mut market = Exchange::new();
        let seller_addr = create_test_address("1");
        let option = create_test_option(seller_addr.clone());
        
        let result = market.list_option(seller_addr, option);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "User not found");
    }

    #[test]
    fn test_unlist_option_success() {
        let (mut market, seller_addr, _) = setup_market_with_users();
        let option = create_test_option(seller_addr.clone());
        
        let listing_id = market.list_option(seller_addr.clone(), option).unwrap();
        
        let result = market.unlist_option(listing_id, seller_addr.clone());
        assert!(result.is_ok());
        
        // Listing should be removed
        assert!(!market.listings.contains_key(&listing_id));
        
        // Collateral should be returned (1 BTC for CALL option)
        let seller = market.users.get(&seller_addr).unwrap();
        assert_eq!(seller.get_balance(&Asset::BTC), 10.0); // Back to original
        assert_eq!(market.escrow_user.get_balance(&Asset::BTC), 0.0);
    }

    #[test]
    fn test_unlist_option_not_owner() {
        let (mut market, seller_addr, buyer_addr) = setup_market_with_users();
        let option = create_test_option(seller_addr.clone());
        
        let listing_id = market.list_option(seller_addr, option).unwrap();
        
        let result = market.unlist_option(listing_id, buyer_addr);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Only the seller can unlist this option");
    }

    #[test]
    fn test_purchase_option_success() {
        let (mut market, seller_addr, buyer_addr) = setup_market_with_users();
        let option = create_test_option(seller_addr.clone());
        
        let listing_id = market.list_option(seller_addr.clone(), option).unwrap();
        
        let result = market.purchase_option(listing_id, buyer_addr.clone());
        assert!(result.is_ok());
        
        // Check option has beneficiary
        let listing = market.listings.get(&listing_id).unwrap();
        assert_eq!(listing.beneficiary_address, Some(buyer_addr.clone()));
        
        // Check payment flows - premium is 50000 (500 * 100), fee is 50 (50000 * 0.001)
        let buyer = market.users.get(&buyer_addr).unwrap();
        assert_eq!(buyer.get_balance(&Asset::USDT), 49950.0); // 100k - premium - fee
        
        let seller = market.users.get(&seller_addr).unwrap();
        assert_eq!(seller.get_balance(&Asset::USDT), 20049950.0); // 20M + premium - fee
        assert_eq!(seller.get_balance(&Asset::BTC), 9.0); // 10 - 1 BTC collateral still in escrow
        
        // Escrow should have BTC collateral and USDT fees
        assert_eq!(market.escrow_user.get_balance(&Asset::BTC), 1.0); // BTC collateral
        assert_eq!(market.escrow_user.get_balance(&Asset::USDT), 100.0); // 2 fees (50 each)
    }

    #[test]
    fn test_purchase_option_insufficient_funds() {
        let (mut market, seller_addr, buyer_addr) = setup_market_with_users();
        
        // Reduce buyer's balance
        let buyer = market.users.get_mut(&buyer_addr).unwrap();
        buyer.deduct_asset(&Asset::USDT, 99000.0).unwrap(); // Leave only 1000 USDT
        
        let option = create_test_option(seller_addr.clone());
        let listing_id = market.list_option(seller_addr, option).unwrap();
        
        let result = market.purchase_option(listing_id, buyer_addr);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Buyer doesn't have enough quote balance to purchase option");
    }

    #[test]
    fn test_purchase_option_listing_not_found() {
        let (mut market, _, buyer_addr) = setup_market_with_users();
        
        let result = market.purchase_option(999, buyer_addr);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Listing not found");
    }

    #[test]
    fn test_unlist_option_after_purchase() {
        let (mut market, seller_addr, buyer_addr) = setup_market_with_users();
        let option = create_test_option(seller_addr.clone());
        
        let listing_id = market.list_option(seller_addr.clone(), option).unwrap();
        market.purchase_option(listing_id, buyer_addr).unwrap();
        
        let result = market.unlist_option(listing_id, seller_addr);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Option has been acquired, cannot unlist");
    }

    #[test]
    fn test_multiple_listings() {
        let (mut market, seller_addr, _) = setup_market_with_users();
        
        let option1 = create_test_option(seller_addr.clone());
        let mut option2 = create_test_option(seller_addr.clone());
        option2.strike_price = 60000.0;
        
        let listing_id1 = market.list_option(seller_addr.clone(), option1).unwrap();
        let listing_id2 = market.list_option(seller_addr.clone(), option2).unwrap();
        
        assert_eq!(listing_id1, 1);
        assert_eq!(listing_id2, 2);
        assert_eq!(market.listings.len(), 2);
        
        // Different strike prices
        assert_eq!(market.listings.get(&listing_id1).unwrap().strike_price, 50000.0);
        assert_eq!(market.listings.get(&listing_id2).unwrap().strike_price, 60000.0);
    }
}
