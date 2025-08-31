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