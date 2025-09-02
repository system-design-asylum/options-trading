use options_trading::{Address, Asset, User};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_address(suffix: &str) -> Address {
        let base = "0x123456789012345678901234567890123456789";
        let full_address = format!("{}{}", base, suffix);
        Address::from(&full_address).unwrap()
    }

    #[test]
    fn test_user_creation() {
        let address = create_test_address("0");
        let user = User::new(address.clone());

        assert_eq!(user.address, address);
        assert!(!user.balances.is_empty());

        // Should have all assets initialized to 0
        assert_eq!(user.get_balance(&Asset::BTC), 0.0);
        assert_eq!(user.get_balance(&Asset::ETH), 0.0);
        assert_eq!(user.get_balance(&Asset::SOL), 0.0);
        assert_eq!(user.get_balance(&Asset::USDT), 0.0);
    }

    #[test]
    fn test_add_asset() {
        let address = create_test_address("1");
        let mut user = User::new(address);

        user.add_asset(&Asset::BTC, 1.5).unwrap();
        assert_eq!(user.get_balance(&Asset::BTC), 1.5);

        // Add more to the same asset
        user.add_asset(&Asset::BTC, 0.5).unwrap();
        assert_eq!(user.get_balance(&Asset::BTC), 2.0);

        // Add different asset
        user.add_asset(&Asset::ETH, 10.0).unwrap();
        assert_eq!(user.get_balance(&Asset::ETH), 10.0);
        assert_eq!(user.get_balance(&Asset::BTC), 2.0); // Should not affect BTC
    }

    #[test]
    fn test_deduct_asset_success() {
        let address = create_test_address("2");
        let mut user = User::new(address);

        user.add_asset(&Asset::BTC, 5.0).unwrap();

        let result = user.deduct_asset(&Asset::BTC, 2.0);
        assert!(result.is_ok());
        assert_eq!(user.get_balance(&Asset::BTC), 3.0);
    }

    #[test]
    fn test_deduct_asset_insufficient_balance() {
        let address = create_test_address("3");
        let mut user = User::new(address);

        user.add_asset(&Asset::BTC, 1.0).unwrap();

        let result = user.deduct_asset(&Asset::BTC, 2.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient BTC balance");
        assert_eq!(user.get_balance(&Asset::BTC), 1.0); // Balance should remain unchanged
    }

    #[test]
    fn test_deduct_asset_zero_balance() {
        let address = create_test_address("4");
        let mut user = User::new(address);

        let result = user.deduct_asset(&Asset::BTC, 1.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Insufficient BTC balance");
    }

    #[test]
    fn test_deduct_asset_exact_balance() {
        let address = create_test_address("5");
        let mut user = User::new(address);

        user.add_asset(&Asset::ETH, 10.0).unwrap();

        let result = user.deduct_asset(&Asset::ETH, 10.0);
        assert!(result.is_ok());
        assert_eq!(user.get_balance(&Asset::ETH), 0.0);
    }

    #[test]
    fn test_get_balance_nonexistent_asset() {
        let address = create_test_address("6");
        let user = User::new(address);

        // All assets should be initialized to 0.0
        assert_eq!(
            user.get_balance(&Asset::OTHER("NONEXISTENT".to_string())),
            0.0
        );
    }

    #[test]
    fn test_multiple_asset_operations() {
        let address = create_test_address("7");
        let mut user = User::new(address);

        // Add multiple assets
        user.add_asset(&Asset::BTC, 2.0).unwrap();
        user.add_asset(&Asset::ETH, 50.0).unwrap();
        user.add_asset(&Asset::USDT, 1000.0).unwrap();

        // Verify balances
        assert_eq!(user.get_balance(&Asset::BTC), 2.0);
        assert_eq!(user.get_balance(&Asset::ETH), 50.0);
        assert_eq!(user.get_balance(&Asset::USDT), 1000.0);

        // Deduct from multiple assets
        user.deduct_asset(&Asset::BTC, 0.5).unwrap();
        user.deduct_asset(&Asset::USDT, 500.0).unwrap();

        // Verify final balances
        assert_eq!(user.get_balance(&Asset::BTC), 1.5);
        assert_eq!(user.get_balance(&Asset::ETH), 50.0); // unchanged
        assert_eq!(user.get_balance(&Asset::USDT), 500.0);
    }

    #[test]
    fn test_user_clone() {
        let address = create_test_address("8");
        let mut user = User::new(address.clone());
        user.add_asset(&Asset::BTC, 1.0).unwrap();

        let cloned_user = user.clone();
        assert_eq!(cloned_user.address, address);
        assert_eq!(cloned_user.get_balance(&Asset::BTC), 1.0);

        // Modifying original should not affect clone
        user.add_asset(&Asset::BTC, 1.0).unwrap();
        assert_eq!(user.get_balance(&Asset::BTC), 2.0);
        assert_eq!(cloned_user.get_balance(&Asset::BTC), 1.0);
    }
}
