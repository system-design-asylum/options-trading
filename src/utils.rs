use crate::types::Address;

pub fn are_addresses_equal(address1: &Address, address2: &Address) -> bool {
    address1.to_string().to_lowercase() == address2.to_string().to_lowercase()
}