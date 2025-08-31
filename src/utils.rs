use crate::address::Address;

pub fn are_addresses_equal(address1: &Address, address2: &Address) -> bool {
    address1.to_normalized() == address2.to_normalized()
}
