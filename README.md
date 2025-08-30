# Options Trading - risky, high leverage trading for power traders
**Concept:** Buyer acquires the right, but not the obligation to buy/sell the underlying assets defined in the options contract at any given time over the agreed upon period.


**Risks:** Theoretical unlimited gain but limited loss for buyer | Theoretical unlimited loss but limited gain for seller

## A Rust-based options trading simulation engine built for learning both Rust programming and options trading concepts.

## Project Structure

```
src/
├── lib.rs          # Main library exports
├── main.rs         # Entry point and simulation runner  
├── types.rs        # Core data types (Asset, ListingType, etc.)
├── user.rs         # User account management
├── option.rs       # Options contract logic
├── market.rs       # Market operations and order matching
├── simulation.rs   # Trading bot strategies and simulation
└── utils.rs        # Utility functions
```

### Implementation
- **Multi-user trading environment** with cash and asset management
- **Options contracts**: CALL and PUT options with expiration dates
- **Market operations**: List, unlist, and buy options
- **Trading strategies**: Aggressive seller/buyer, balanced, market maker
- **Real-time simulation** with portfolio tracking
- **Premium pricing** with 100x multiplier (industry standard)
- **Asset reservation** for CALL options to ensure delivery capability

## Running the Simulation

```bash
# Run the default 20-round simulation
cargo run

# Build the library for use in other projects
cargo build
```

## Backlog

The simulation shows:
- **User portfolios** with cash and asset balances
- **Active listings** with strike prices, premiums, and sellers
- **Trading activity** as bots execute strategies
- **Market statistics** including total value and listing counts

## Learning Areas

### Rust Programming
- **Ownership & Borrowing**: See how data flows between functions (help!)
- **Error Handling**: Result<T, E> patterns throughout (pls help!)
- **Modules**: Clean separation of concerns

Happy coding! 🦀
