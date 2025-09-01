use options_trading::{ListingOption, Asset, ListingType, Address};
use chrono::{Utc, Duration};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_address(suffix: &str) -> Address {
        let base = "0x123456789012345678901234567890123456789";
        let full_address = format!("{}{}", base, suffix);
        Address::from(&full_address).unwrap()
    }

    fn create_test_option() -> ListingOption {
        ListingOption {
            listing_id: 1,
            base_asset: Asset::BTC,
            quote_asset: Asset::USDT,
            listing_type: ListingType::CALL,
            strike_price: 50000.0,
            ask_price: 500.0,
            bid_price: 490.0,
            expiration_time: Utc::now() + Duration::days(30),
            grantor_address: create_test_address("1"),
            beneficiary_address: None,
        }
    }

    #[test]
    fn test_listing_option_creation() {
        let option = create_test_option();
        
        assert_eq!(option.listing_id, 1);
        assert_eq!(option.base_asset, Asset::BTC);
        assert_eq!(option.quote_asset, Asset::USDT);
        assert_eq!(option.listing_type, ListingType::CALL);
        assert_eq!(option.strike_price, 50000.0);
        assert_eq!(option.ask_price, 500.0);
        assert_eq!(option.bid_price, 490.0);
        assert!(option.beneficiary_address.is_none());
    }

    #[test]
    fn test_get_premium_price() {
        let option = create_test_option();
        
        // Premium price should be ask_price * 100
        assert_eq!(option.get_premium_price(), 50000.0); // 500.0 * 100
    }

    #[test]
    fn test_get_collateral_price() {
        let option = create_test_option();
        
        // Collateral price should be strike_price * 100
        assert_eq!(option.get_collateral_price(), 5000000.0); // 50000.0 * 100
    }

    #[test]
    fn test_different_asset_combinations() {
        let mut option = create_test_option();
        
        // Test BTC/ETH pair
        option.base_asset = Asset::BTC;
        option.quote_asset = Asset::ETH;
        option.strike_price = 15.0; // 1 BTC = 15 ETH
        option.ask_price = 1.5;
        
        assert_eq!(option.get_premium_price(), 150.0); // 1.5 * 100
        assert_eq!(option.get_collateral_price(), 1500.0); // 15.0 * 100
    }

    #[test]
    fn test_put_option() {
        let mut option = create_test_option();
        option.listing_type = ListingType::PUT;
        
        assert_eq!(option.listing_type, ListingType::PUT);
        // Premium and collateral calculations should remain the same
        assert_eq!(option.get_premium_price(), 50000.0);
        assert_eq!(option.get_collateral_price(), 5000000.0);
    }

    #[test]
    fn test_option_with_beneficiary() {
        let mut option = create_test_option();
        let beneficiary = create_test_address("2");
        option.beneficiary_address = Some(beneficiary.clone());
        
        assert!(option.beneficiary_address.is_some());
        assert_eq!(option.beneficiary_address.unwrap(), beneficiary);
    }

    #[test]
    fn test_option_clone() {
        let option = create_test_option();
        let cloned_option = option.clone();
        
        assert_eq!(option.listing_id, cloned_option.listing_id);
        assert_eq!(option.base_asset, cloned_option.base_asset);
        assert_eq!(option.quote_asset, cloned_option.quote_asset);
        assert_eq!(option.listing_type, cloned_option.listing_type);
        assert_eq!(option.strike_price, cloned_option.strike_price);
        assert_eq!(option.ask_price, cloned_option.ask_price);
        assert_eq!(option.grantor_address, cloned_option.grantor_address);
    }

    #[test]
    fn test_zero_prices() {
        let mut option = create_test_option();
        option.ask_price = 0.0;
        option.strike_price = 0.0;
        
        assert_eq!(option.get_premium_price(), 0.0);
        assert_eq!(option.get_collateral_price(), 0.0);
    }

    #[test]
    fn test_decimal_prices() {
        let mut option = create_test_option();
        option.ask_price = 1.23;
        option.strike_price = 45678.90;
        
        assert_eq!(option.get_premium_price(), 123.0); // 1.23 * 100
        assert_eq!(option.get_collateral_price(), 4567890.0); // 45678.90 * 100
    }

    #[test]
    fn test_expiration_future() {
        let option = create_test_option();
        
        // Expiration should be in the future (30 days from now)
        assert!(option.expiration_time > Utc::now());
    }

    #[test]
    fn test_debug_format() {
        let option: ListingOption = create_test_option();
        let debug_string = format!("{:?}", option);
        
        // Should contain key information
        assert!(debug_string.contains("ListingOption"));
        assert!(debug_string.contains("BTC"));
        assert!(debug_string.contains("USDT"));
        assert!(debug_string.contains("CALL"));
    }
}
