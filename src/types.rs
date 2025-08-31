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
