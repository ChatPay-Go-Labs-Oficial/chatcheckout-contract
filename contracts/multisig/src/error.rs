use soroban_sdk::contracterror;

#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracterror]
#[repr(u32)]
pub enum MultisigError {
    /// Contract is already initialized
    AlreadyInitialized = 1,

    /// Contract is not initialized
    NotInitialized = 2,

    /// Unauthorized: caller is not the multisig itself
    Unauthorized = 3,

    /// Invalid threshold (0, or greater than number of signers)
    InvalidThreshold = 4,

    /// Signer public key already registered
    SignerAlreadyExists = 5,

    /// Signer public key not found
    SignerNotFound = 6,

    /// Not enough valid signatures to meet threshold
    InsufficientSignatures = 7,

    /// Signatures are not in strictly ascending order by public key
    InvalidSignatureOrder = 8,

    /// Maximum number of signers reached
    TooManySigners = 9,

    /// Removing this signer would reduce signers below threshold
    ThresholdExceedsSigners = 10,

    /// Signature provided for an address that is not a registered signer
    UnknownSigner = 11,
}
