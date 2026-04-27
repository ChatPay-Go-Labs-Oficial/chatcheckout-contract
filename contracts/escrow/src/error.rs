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

    /// Invalid signature for meta-transaction
    InvalidSignature = 13,

    /// Nonce mismatch - replay attack detected
    InvalidNonce = 14,

    /// Signature expired
    SignatureExpired = 15,

    /// Invalid function selector for meta-transaction
    InvalidFunctionSelector = 16,

    /// Counter overflow (too many escrows)
    CounterOverflow = 17,

    /// Token not in allowed list
    TokenNotAllowed = 18,

    /// Escrow is not in disputed status
    NotDisputed = 19,

    /// Escrow is already disputed
    AlreadyDisputed = 20,

    /// No dispute to resolve (no proposals exist)
    NoDisputeToResolve = 21,

    /// Dispute already resolved
    DisputeAlreadyResolved = 22,

    /// Both parties must agree on resolution
    BothPartiesMustAgree = 23,

    /// Party has already proposed resolution
    AlreadyProposed = 24,

    /// Seller cannot initiate a dispute during the buyer's guarantee period
    DisputeNotAllowed = 25,

    /// Buyer and seller cannot be the same address
    InvalidSeller = 26,

    /// Allowed token list is at its maximum capacity
    TokenLimitReached = 27,
}
