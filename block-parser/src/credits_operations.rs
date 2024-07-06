use super::*;

// The operations on `credits.aleo` that need to be tracked during block processing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CreditsOperations {
    // The `bond_public` operation.
    BondPublic {
        id: String,
        validator: String,
        withdrawal: String,
        amount: u64,
    },
    // The `claim_unbond_public` operation.
    ClaimUnbondPublic {
        id: String,
        staker: String,
    },
    // The `unbond_public` operation.
    UnbondPublic {
        id: String,
        staker: String,
        amount: u64,
    },
}
