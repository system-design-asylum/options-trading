use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Asset {
    BTC,
    ETH,
    SOL,
    APPLE,
    OTHER(String), // changed from &str to owned String to avoid lifetime issues

    // fiat representation assets
    USDT,
    USDC,
    VNDT,
    VNDC,
}

impl std::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Asset::BTC => write!(f, "BTC"),
            Asset::ETH => write!(f, "ETH"),
            Asset::SOL => write!(f, "SOL"),
            Asset::APPLE => write!(f, "APPLE"),
            Asset::USDT => write!(f, "USDT"),
            Asset::USDC => write!(f, "USDC"),
            Asset::VNDT => write!(f, "VNDT"),
            Asset::VNDC => write!(f, "VNDC"),
            Asset::OTHER(s) => write!(f, "{}", s), // handle OTHER variant
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListingType {
    CALL,
    PUT,
}

impl std::fmt::Display for ListingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ListingType::CALL => write!(f, "CALL"),
            ListingType::PUT => write!(f, "PUT"),
        }
    }
}

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
}
