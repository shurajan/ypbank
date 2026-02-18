/// A single financial transaction record.
///
/// Transactions are the core data model of the `ypbank` crate and can be
/// encoded/decoded using codecs such as [`Csv`], [`Txt`], or [`Bin`].
///
/// ## CSV Field Mapping
///
/// The CSV representation uses the following columns:
///
/// - `TX_ID` — 64-bit integer, unique transaction identifier.
/// - `TX_TYPE` — transaction type (`DEPOSIT`, `TRANSFER`, `WITHDRAWAL`).
/// - `FROM_USER_ID` — sender user ID. For system deposits (`DEPOSIT`),
///   this value may be `0`.
/// - `TO_USER_ID` — receiver user ID. For system withdrawals (`WITHDRAWAL`),
///   this value may be `0`.
/// - `AMOUNT` — transaction amount in the smallest currency unit
///   (for example, cents).
/// - `TIMESTAMP` — Unix timestamp in milliseconds since epoch.
/// - `STATUS` — transaction status (`SUCCESS`, `FAILURE`, `PENDING`).
/// - `DESCRIPTION` — textual description. This is always the last field
///   in the row and must be enclosed in double quotes (`"`).
///
/// ## Example
///
/// ```
/// use parser::{Transaction, TxType, TxStatus};
///
/// let tx = Transaction {
///     tx_id: 1001,
///     tx_type: TxType::Deposit,
///     from_user_id: 0,
///     to_user_id: 501,
///     amount: 50000,
///     timestamp: 1672531200000,
///     status: TxStatus::Success,
///     description: "Initial funding".to_string(),
/// };
///
/// assert_eq!(tx.tx_id, 1001);
/// ```

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    /// Unique transaction identifier (`TX_ID`).
    pub tx_id: u64,

    /// Transaction type (`TX_TYPE`).
    pub tx_type: TxType,

    /// Sender user identifier (`FROM_USER_ID`).
    ///
    /// For system deposits, this may be `0`.
    pub from_user_id: u64,

    /// Receiver user identifier (`TO_USER_ID`).
    ///
    /// For system withdrawals, this may be `0`.
    pub to_user_id: u64,

    /// Transaction amount in the smallest currency unit (`AMOUNT`).
    pub amount: u64,

    /// Unix timestamp in milliseconds (`TIMESTAMP`).
    pub timestamp: u64,

    /// Transaction execution status (`STATUS`).
    pub status: TxStatus,

    /// Human-readable transaction description (`DESCRIPTION`).
    ///
    /// In CSV or TXT format, this field is always quoted.
    pub description: String,
}

/// Transaction type (`TX_TYPE`).
/// ## Supported Values
/// - `DEPOSIT` — funds added to a user account
/// - `TRANSFER` — funds transferred between users
/// - `WITHDRAWAL` — funds withdrawn from a user account
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxType {
    /// System deposit into an account.
    Deposit,

    /// Transfer between two user accounts.
    Transfer,

    /// System withdrawal from an account.
    Withdrawal,
}
impl TxType {
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "DEPOSIT" => Some(TxType::Deposit),
            "TRANSFER" => Some(TxType::Transfer),
            "WITHDRAWAL" => Some(TxType::Withdrawal),
            _ => None,
        }
    }

    pub(crate) fn from(val: u8) -> Option<Self> {
        match val {
            0 => Some(TxType::Deposit),
            1 => Some(TxType::Transfer),
            2 => Some(TxType::Withdrawal),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            TxType::Deposit => "DEPOSIT",
            TxType::Transfer => "TRANSFER",
            TxType::Withdrawal => "WITHDRAWAL",
        }
    }

    pub(crate) fn to_byte(&self) -> u8 {
        match self {
            TxType::Deposit => 0u8,
            TxType::Transfer => 1u8,
            TxType::Withdrawal => 2u8,
        }
    }
}

impl std::fmt::Display for TxType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Transaction execution status (`STATUS`).
/// ## Supported Values
/// - `SUCCESS` — the transaction was completed successfully
/// - `FAILURE` — the transaction failed to execute
/// - `PENDING` — the transaction is still being processed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxStatus {
    /// Transaction completed successfully.
    Success,
    /// Transaction execution failed.
    Failure,
    /// Transaction is still pending.
    Pending,
}

impl TxStatus {
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "SUCCESS" => Some(TxStatus::Success),
            "FAILURE" => Some(TxStatus::Failure),
            "PENDING" => Some(TxStatus::Pending),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            TxStatus::Success => "SUCCESS",
            TxStatus::Failure => "FAILURE",
            TxStatus::Pending => "PENDING",
        }
    }

    pub(crate) fn from(val: u8) -> Option<Self> {
        match val {
            0 => Some(TxStatus::Success),
            1 => Some(TxStatus::Failure),
            2 => Some(TxStatus::Pending),
            _ => None,
        }
    }

    pub(crate) fn to_byte(&self) -> u8 {
        match self {
            TxStatus::Success => 0u8,
            TxStatus::Failure => 1u8,
            TxStatus::Pending => 2u8,
        }
    }
}

impl std::fmt::Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tx_type_ok() {
        assert_eq!(TxType::parse("DEPOSIT").unwrap(), TxType::Deposit);
        assert_eq!(TxType::parse("TRANSFER").unwrap(), TxType::Transfer);
        assert_eq!(TxType::parse("WITHDRAWAL").unwrap(), TxType::Withdrawal);
    }

    #[test]
    fn test_parse_tx_type_err() {
        assert_eq!(TxType::parse("Deposit"), None);
        assert_eq!(TxType::parse(""), None);
        assert_eq!(TxType::parse("ABC"), None);
    }

    #[test]
    fn test_parse_tx_status_ok() {
        assert_eq!(TxStatus::parse("SUCCESS").unwrap(), TxStatus::Success);
        assert_eq!(TxStatus::parse("FAILURE").unwrap(), TxStatus::Failure);
        assert_eq!(TxStatus::parse("PENDING").unwrap(), TxStatus::Pending);
    }

    #[test]
    fn test_parse_tx_status_err() {
        assert_eq!(TxStatus::parse("Success"), None);
        assert_eq!(TxStatus::parse(""), None);
        assert_eq!(TxStatus::parse("ABC"), None);
    }
}
