use std::io::{Cursor, Read, Write};

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub tx_id: u64,
    pub tx_type: TxType,
    pub from_user_id: u64,
    pub to_user_id: u64,
    pub amount: u64,
    pub timestamp: u64,
    pub status: TxStatus,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxType {
    Deposit,
    Transfer,
    Withdrawal,
}

impl TxType {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "DEPOSIT" => Some(TxType::Deposit),
            "TRANSFER" => Some(TxType::Transfer),
            "WITHDRAWAL" => Some(TxType::Withdrawal),
            _ => None,
        }
    }

    pub fn from(val: u8) -> Option<Self> {
        match val {
            0 => Some(TxType::Deposit),
            1 => Some(TxType::Transfer),
            2 => Some(TxType::Withdrawal),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxStatus {
    Success,
    Failure,
    Pending,
}

impl TxStatus {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "SUCCESS" => Some(TxStatus::Success),
            "FAILURE" => Some(TxStatus::Failure),
            "PENDING" => Some(TxStatus::Pending),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TxStatus::Success => "SUCCESS",
            TxStatus::Failure => "FAILURE",
            TxStatus::Pending => "PENDING",
        }
    }

    pub fn from(val: u8) -> Option<Self> {
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
