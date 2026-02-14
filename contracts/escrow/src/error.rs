use soroban_sdk::contracterror;

/// Custom error types for the escrow contract
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracterror]
#[repr(u32)]
pub enum EscrowError {
    /// Contract is already initialized
    AlreadyInitialized = 1,

    /// Contract configuration not found
    ConfigNotInitialized = 2,

    /// Unauthorized access
    Unauthorized = 3,

    /// Escrow not found
    EscrowNotFound = 4,

    /// Invalid amount (must be greater than 0)
    InvalidAmount = 5,

    /// Invalid fee basis points (must be 0-10000)
    InvalidFeeBps = 6,

    /// Invalid guarantee days (must be 1-36500)
    InvalidGuaranteeDays = 7,

    /// Invalid product ID (cannot be empty)
    InvalidProductId = 8,

    /// Escrow is not in active status
    EscrowNotActive = 9,

    /// Guarantee period not expired
    GuaranteePeriodNotExpired = 10,

    /// Guarantee period already expired
    GuaranteePeriodExpired = 11,

    /// Fee amount exceeds escrow amount
    FeeExceedsAmount = 12,

    /// Escrow is not in disputed status
    EscrowNotDisputed = 13,

    /// Dispute already resolved
    DisputeAlreadyResolved = 14,

    /// Both parties must agree on resolution
    BothPartiesMustAgree = 15,

    /// Cannot resolve own dispute
    CannotResolveOwnDispute = 16,
}
