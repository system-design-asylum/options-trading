use options_trading::types::{ListingType, Address, AddressError};
use options_trading::asset::Asset;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_display() {
        assert_eq!(Asset::BTC.to_string(), "BTC");
        assert_eq!(Asset::ETH.to_string(), "ETH");
        assert_eq!(Asset::SOL.to_string(), "SOL");
        assert_eq!(Asset::APPLE.to_string(), "APPLE");
        assert_eq!(Asset::USDT.to_string(), "USDT");
        assert_eq!(Asset::USDC.to_string(), "USDC");
        assert_eq!(Asset::VNDT.to_string(), "VNDT");
        assert_eq!(Asset::VNDC.to_string(), "VNDC");
        assert_eq!(Asset::OTHER("CUSTOM".to_string()).to_string(), "CUSTOM");
    }

    #[test]
    fn test_asset_equality() {
        assert_eq!(Asset::BTC, Asset::BTC);
        assert_ne!(Asset::BTC, Asset::ETH);
        assert_eq!(Asset::OTHER("TEST".to_string()), Asset::OTHER("TEST".to_string()));
        assert_ne!(Asset::OTHER("TEST1".to_string()), Asset::OTHER("TEST2".to_string()));
    }

    #[test]
    fn test_listing_type_display() {
        assert_eq!(ListingType::CALL.to_string(), "CALL");
        assert_eq!(ListingType::PUT.to_string(), "PUT");
    }

    #[test]
    fn test_listing_type_equality() {
        assert_eq!(ListingType::CALL, ListingType::CALL);
        assert_eq!(ListingType::PUT, ListingType::PUT);
        assert_ne!(ListingType::CALL, ListingType::PUT);
    }

    #[test]
    fn test_address_valid() {
        let valid_address = "0x1234567890123456789012345678901234567890";
        let address = Address::from(valid_address);
        assert!(address.is_ok());
        
        let addr = address.unwrap();
        assert_eq!(addr.to_string(), valid_address);
    }

    #[test]
    fn test_address_invalid_length() {
        let short_address = "0x123456789";
        let long_address = "0x123456789012345678901234567890123456789012345";
        
        assert_eq!(Address::from(short_address), Err(AddressError::InvalidLength));
        assert_eq!(Address::from(long_address), Err(AddressError::InvalidLength));
    }

    #[test]
    fn test_address_equality() {
        let addr1 = Address::from("0x1234567890123456789012345678901234567890").unwrap();
        let addr2 = Address::from("0x1234567890123456789012345678901234567890").unwrap();
        let addr3 = Address::from("0x1234567890123456789012345678901234567891").unwrap();
        
        assert!(addr1.is_equal_to(&addr2));
        assert!(!addr1.is_equal_to(&addr3));
    }

    #[test]
    fn test_address_hash_eq() {
        use std::collections::HashMap;
        
        let addr1 = Address::from("0x1234567890123456789012345678901234567890").unwrap();
        let addr2 = Address::from("0x1234567890123456789012345678901234567890").unwrap();
        
        let mut map = HashMap::new();
        map.insert(addr1, "value1");
        
        // Should be able to retrieve using addr2 since they're equal
        assert_eq!(map.get(&addr2), Some(&"value1"));
    }
}
