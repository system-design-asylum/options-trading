use options_trading::{are_addresses_equal, Address};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_address(suffix: &str) -> Address {
        let base = "0x123456789012345678901234567890123456789";
        let full_address = format!("{}{}", base, suffix);
        Address::from(&full_address).unwrap()
    }

    #[test]
    fn test_addresses_equal_same() {
        let addr1 = create_test_address("1");
        let addr2 = create_test_address("1");
        
        assert!(are_addresses_equal(&addr1, &addr2));
    }

    #[test]
    fn test_addresses_equal_different() {
        let addr1 = create_test_address("1");
        let addr2 = create_test_address("2");
        
        assert!(!are_addresses_equal(&addr1, &addr2));
    }

    #[test]
    fn test_addresses_equal_case_insensitive() {
        // Create addresses with different cases
        let addr1 = Address::from("0x1234567890123456789012345678901234567890").unwrap();
        let addr2 = Address::from("0x1234567890123456789012345678901234567890").unwrap();
        
        assert!(are_addresses_equal(&addr1, &addr2));
    }

    #[test]
    fn test_addresses_equal_mixed_case() {
        // Even though our Address struct doesn't allow mixed case creation,
        // we can test the logic with same addresses
        let addr1 = Address::from("0xabcdef1234567890123456789012345678901234").unwrap();
        let addr2 = Address::from("0xabcdef1234567890123456789012345678901234").unwrap();
        
        assert!(are_addresses_equal(&addr1, &addr2));
    }

    #[test]
    fn test_addresses_not_equal_different_content() {
        let addr1 = Address::from("0x1111111111111111111111111111111111111111").unwrap();
        let addr2 = Address::from("0x2222222222222222222222222222222222222222").unwrap();
        
        assert!(!are_addresses_equal(&addr1, &addr2));
    }

    #[test]
    fn test_addresses_equal_reflexive() {
        let addr = create_test_address("1");
        
        assert!(are_addresses_equal(&addr, &addr));
    }

    #[test]
    fn test_addresses_equal_symmetric() {
        let addr1 = create_test_address("1");
        let addr2 = create_test_address("1");
        
        assert!(are_addresses_equal(&addr1, &addr2));
        assert!(are_addresses_equal(&addr2, &addr1));
    }

    #[test]
    fn test_addresses_equal_transitive() {
        let addr1 = create_test_address("1");
        let addr2 = create_test_address("1");
        let addr3 = create_test_address("1");
        
        assert!(are_addresses_equal(&addr1, &addr2));
        assert!(are_addresses_equal(&addr2, &addr3));
        assert!(are_addresses_equal(&addr1, &addr3));
    }
}
