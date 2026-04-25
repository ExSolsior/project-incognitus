/// Order side for limit/market orders.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum OrderSide {
    Bid = 0,
    Ask = 1,
}

/// Order type flags for instruction args and ledger entries.
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum OrderType {
    Limit = 0,
    Market = 1,
}

/// Generic account version used across state layouts.
pub const ACCOUNT_VERSION: u8 = 0;
