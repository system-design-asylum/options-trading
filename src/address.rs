
// Define a custom type for address for clarity
#[derive(Debug, PartialEq)]
pub enum AddressError {
    InvalidLength,
}

// A typed string that guarantees a length of 42.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Address(String);

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Address {
    pub fn from(s: &str) -> Result<Self, AddressError> {
        if s.len() == 42 {
            Ok(Address(s.to_string()))
        } else {
            Err(AddressError::InvalidLength)
        }
    }

    pub fn is_equal_to(&self, another_address: &Address) -> bool {
        self.0 == another_address.0
    }

	pub fn to_normalized(&self) -> Address {
		return Address(self.0.to_lowercase());
	}
}
