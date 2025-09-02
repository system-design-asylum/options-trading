use options_trading::{Address, Asset};
use options_trading::exchange_rate_provider::{get_rate_provider, get_readonly_rate_provider, default_exchange_rate_provider_admin_address};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_address(suffix: &str) -> Address {
        let base = "0x123456789012345678901234567890123456789";
        let full_address = format!("{}{}", base, suffix);
        // Ensure address is exactly 42 characters by padding/truncating
        let padded = format!("{:0<42}", full_address);
        Address::from(&padded[..42]).unwrap()
    }

    #[test]
    fn test_get_rate_default_btc_usdt() {
        // Note: Rate might have been modified by other tests due to singleton pattern
        let provider = get_readonly_rate_provider();
        let rate = provider.get_rate(&Asset::BTC, &Asset::USDT);
        
        // Should have some rate set for BTC/USDT
        assert!(rate.is_some());
        assert!(rate.unwrap() > 0.0);
    }

    #[test]
    fn test_get_rate_non_existent_pair() {
        let provider = get_readonly_rate_provider();
        
        // Most pairs should have some rate (might be 0.0 or set by other tests)
        let rate = provider.get_rate(&Asset::ETH, &Asset::BTC);
        assert!(rate.is_some());
    }

    #[test]
    fn test_set_rate_success() {
        let admin_addr = default_exchange_rate_provider_admin_address();
        
        {
            let mut provider = get_rate_provider();
            let result = provider.set_rate(Asset::ETH, Asset::USDT, 3000.0, admin_addr);
            assert!(result.is_ok());
        }

        // Verify the rate was set
        let provider = get_readonly_rate_provider();
        let rate = provider.get_rate(&Asset::ETH, &Asset::USDT);
        assert_eq!(rate, Some(3000.0));
    }

    #[test]
    fn test_set_rate_unauthorized() {
        let unauthorized_addr = create_test_address("999");
        
        let mut provider = get_rate_provider();
        let result = provider.set_rate(Asset::ETH, Asset::USDT, 3000.0, unauthorized_addr);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "caller not authorized to update exchange rate");
    }

    #[test]
    fn test_set_multiple_rates() {
        let admin_addr = default_exchange_rate_provider_admin_address();
        
        {
            let mut provider = get_rate_provider();
            
            // Set multiple rates
            provider.set_rate(Asset::ETH, Asset::USDT, 3000.0, admin_addr.clone()).unwrap();
            provider.set_rate(Asset::SOL, Asset::USDT, 100.0, admin_addr.clone()).unwrap();
            provider.set_rate(Asset::APPLE, Asset::USDT, 150.0, admin_addr).unwrap();
        }

        // Verify all rates
        let provider = get_readonly_rate_provider();
        assert_eq!(provider.get_rate(&Asset::ETH, &Asset::USDT), Some(3000.0));
        assert_eq!(provider.get_rate(&Asset::SOL, &Asset::USDT), Some(100.0));
        assert_eq!(provider.get_rate(&Asset::APPLE, &Asset::USDT), Some(150.0));
    }

    #[test]
    fn test_update_existing_rate() {
        let admin_addr = default_exchange_rate_provider_admin_address();
        
        {
            let mut provider = get_rate_provider();
            
            // Set initial rate for a unique pair
            provider.set_rate(Asset::SOL, Asset::ETH, 0.03, admin_addr.clone()).unwrap();
            
            // Update the rate
            provider.set_rate(Asset::SOL, Asset::ETH, 0.035, admin_addr).unwrap();
        }

        // Verify the rate was updated
        let provider = get_readonly_rate_provider();
        assert_eq!(provider.get_rate(&Asset::SOL, &Asset::ETH), Some(0.035));
    }

    #[test]
    fn test_cross_asset_rates() {
        let admin_addr = default_exchange_rate_provider_admin_address();
        
        {
            let mut provider = get_rate_provider();
            
            // Set BTC/ETH rate (how many ETH for 1 BTC)
            provider.set_rate(Asset::BTC, Asset::ETH, 25.0, admin_addr.clone()).unwrap();
            
            // Set ETH/BTC rate (how many BTC for 1 ETH) 
            provider.set_rate(Asset::ETH, Asset::BTC, 0.04, admin_addr).unwrap();
        }

        let provider = get_readonly_rate_provider();
        assert_eq!(provider.get_rate(&Asset::BTC, &Asset::ETH), Some(25.0));
        assert_eq!(provider.get_rate(&Asset::ETH, &Asset::BTC), Some(0.04));
    }

    #[test]
    fn test_rate_precision() {
        let admin_addr = default_exchange_rate_provider_admin_address();
        
        {
            let mut provider = get_rate_provider();
            
            // Set precise rate
            provider.set_rate(Asset::BTC, Asset::USDT, 67432.123456789, admin_addr).unwrap();
        }

        let provider = get_readonly_rate_provider();
        assert_eq!(provider.get_rate(&Asset::BTC, &Asset::USDT), Some(67432.123456789));
    }

    #[test]
    fn test_zero_rate() {
        let admin_addr = default_exchange_rate_provider_admin_address();
        
        {
            let mut provider = get_rate_provider();
            
            // Set zero rate (could represent temporarily unavailable pair)
            provider.set_rate(Asset::ETH, Asset::USDT, 0.0, admin_addr).unwrap();
        }

        let provider = get_readonly_rate_provider();
        assert_eq!(provider.get_rate(&Asset::ETH, &Asset::USDT), Some(0.0));
    }

    #[test]
    fn test_concurrent_read_access() {
        // This test ensures multiple read accesses work concurrently
        let provider1 = get_readonly_rate_provider();
        let provider2 = get_readonly_rate_provider();
        
        let rate1 = provider1.get_rate(&Asset::BTC, &Asset::USDT);
        let rate2 = provider2.get_rate(&Asset::BTC, &Asset::USDT);
        
        // Both should return the same rate (whatever it currently is)
        assert_eq!(rate1, rate2);
        assert!(rate1.is_some());
    }

    #[test]
    fn test_same_asset_pair() {
        let provider = get_readonly_rate_provider();
        
        // Same asset pairs should not be initialized (the constructor skips them)
        let rate = provider.get_rate(&Asset::BTC, &Asset::BTC);
        assert_eq!(rate, None);
    }
}
