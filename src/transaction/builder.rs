use crate::error::ReaderError;
use crate::schema;
use crate::transaction::{Transaction, TxStatus, TxType};

#[derive(Default)]
pub struct TransactionBuilder {
    tx_id: Option<u64>,
    tx_type: Option<TxType>,
    from_user_id: Option<u64>,
    to_user_id: Option<u64>,
    amount: Option<u64>,
    timestamp: Option<u64>,
    status: Option<TxStatus>,
    description: Option<String>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: &str, value: &str, line_no: usize) -> Result<(), ReaderError> {
        match key {
            schema::TX_ID => {
                if self.tx_id.is_some() {
                    return Err(ReaderError::DuplicateField {
                        line_no,
                        field: key.to_string(),
                    });
                }
                self.tx_id = Some(Self::parse_u64(key, value, line_no)?);
            }
            schema::TX_TYPE => {
                if self.tx_type.is_some() {
                    return Err(ReaderError::DuplicateField {
                        line_no,
                        field: key.to_string(),
                    });
                }
                self.tx_type = Some(Self::parse_tx_type(value, line_no)?);
            }
            schema::FROM_USER_ID => {
                if self.from_user_id.is_some() {
                    return Err(ReaderError::DuplicateField {
                        line_no,
                        field: key.to_string(),
                    });
                }
                self.from_user_id = Some(Self::parse_u64(key, value, line_no)?);
            }
            schema::TO_USER_ID => {
                if self.to_user_id.is_some() {
                    return Err(ReaderError::DuplicateField {
                        line_no,
                        field: key.to_string(),
                    });
                }
                self.to_user_id = Some(Self::parse_u64(key, value, line_no)?);
            }
            schema::AMOUNT => {
                if self.amount.is_some() {
                    return Err(ReaderError::DuplicateField {
                        line_no,
                        field: key.to_string(),
                    });
                }
                self.amount = Some(Self::parse_u64(key, value, line_no)?);
            }
            schema::TIMESTAMP => {
                if self.timestamp.is_some() {
                    return Err(ReaderError::DuplicateField {
                        line_no,
                        field: key.to_string(),
                    });
                }
                self.timestamp = Some(Self::parse_u64(key, value, line_no)?);
            }
            schema::STATUS => {
                if self.status.is_some() {
                    return Err(ReaderError::DuplicateField {
                        line_no,
                        field: key.to_string(),
                    });
                }
                self.status = Some(Self::parse_status(value, line_no)?);
            }
            schema::DESCRIPTION => {
                if self.description.is_some() {
                    return Err(ReaderError::DuplicateField {
                        line_no,
                        field: key.to_string(),
                    });
                }
                self.description = Some(Self::parse_description(value, line_no)?);
            }
            _ => {
                return Err(ReaderError::UnknownField {
                    line_no,
                    field: key.to_string(),
                });
            }
        }
        Ok(())
    }

    pub fn build(self, line_no: usize) -> Result<Transaction, ReaderError> {
        let missing: Vec<String> = [
            (self.tx_id.is_none(), schema::TX_ID),
            (self.tx_type.is_none(), schema::TX_TYPE),
            (self.from_user_id.is_none(), schema::FROM_USER_ID),
            (self.to_user_id.is_none(), schema::TO_USER_ID),
            (self.amount.is_none(), schema::AMOUNT),
            (self.timestamp.is_none(), schema::TIMESTAMP),
            (self.status.is_none(), schema::STATUS),
            (self.description.is_none(), schema::DESCRIPTION),
        ]
        .into_iter()
        .filter(|(is_missing, _)| *is_missing)
        .map(|(_, name)| name.to_string())
        .collect();

        if !missing.is_empty() {
            return Err(ReaderError::MissingFields {
                line_no,
                fields: missing,
            });
        }

        match (
            self.tx_id,
            self.tx_type,
            self.from_user_id,
            self.to_user_id,
            self.amount,
            self.timestamp,
            self.status,
            self.description,
        ) {
            (
                Some(tx_id),
                Some(tx_type),
                Some(from_user_id),
                Some(to_user_id),
                Some(amount),
                Some(timestamp),
                Some(status),
                Some(description),
            ) => Ok(Transaction {
                tx_id,
                tx_type,
                from_user_id,
                to_user_id,
                amount,
                timestamp,
                status,
                description,
            }),
            _ => unreachable!("all fields validated above"),
        }
    }

    fn parse_u64(field: &str, value: &str, line_no: usize) -> Result<u64, ReaderError> {
        value.parse().map_err(|_| ReaderError::InvalidFieldValue {
            line_no,
            field: field.to_string(),
            value: value.to_string(),
        })
    }

    fn parse_tx_type(value: &str, line_no: usize) -> Result<TxType, ReaderError> {
        TxType::parse(value).ok_or_else(|| ReaderError::InvalidFieldValue {
            line_no,
            field: schema::TX_TYPE.to_string(),
            value: value.to_string(),
        })
    }

    fn parse_status(value: &str, line_no: usize) -> Result<TxStatus, ReaderError> {
        TxStatus::parse(value).ok_or_else(|| ReaderError::InvalidFieldValue {
            line_no,
            field: schema::STATUS.to_string(),
            value: value.to_string(),
        })
    }

    fn parse_description(value: &str, line_no: usize) -> Result<String, ReaderError> {
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            Ok(value.to_string())
        } else {
            Err(ReaderError::InvalidRow {
                line_no,
                reason: format!("broken quotes for {}", schema::DESCRIPTION),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::{TxStatus, TxType};

    // ─────────────────────────────────────────────────────────────────────
    // Helpers
    // ─────────────────────────────────────────────────────────────────────
    fn complete_builder() -> TransactionBuilder {
        let mut b = TransactionBuilder::new();
        let _ = b.set("TX_ID", "1234567890", 0);
        let _ = b.set("TX_TYPE", "DEPOSIT", 0);
        let _ = b.set("FROM_USER_ID", "0", 0);
        let _ = b.set("TO_USER_ID", "9876543210", 0);
        let _ = b.set("AMOUNT", "10000", 0);
        let _ = b.set("TIMESTAMP", "1633036800000", 0);
        let _ = b.set("STATUS", "SUCCESS", 0);
        let _ = b.set("DESCRIPTION", "\"Test deposit\"", 0);
        b
    }

    // ─────────────────────────────────────────────────────────────────────
    // Success cases
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn build_complete_transaction() {
        let result = complete_builder().build(0);
        assert!(result.is_ok());

        let tx = match result {
            Ok(tx) => tx,
            Err(_) => return,
        };

        assert_eq!(tx.tx_id, 1234567890);
        assert_eq!(tx.tx_type, TxType::Deposit);
        assert_eq!(tx.from_user_id, 0);
        assert_eq!(tx.to_user_id, 9876543210);
        assert_eq!(tx.amount, 10000);
        assert_eq!(tx.timestamp, 1633036800000);
        assert_eq!(tx.status, TxStatus::Success);
        assert_eq!(tx.description, "\"Test deposit\"");
    }

    #[test]
    fn build_all_tx_types() {
        for (s, expected) in [
            ("DEPOSIT", TxType::Deposit),
            ("TRANSFER", TxType::Transfer),
            ("WITHDRAWAL", TxType::Withdrawal),
        ] {
            let mut b = TransactionBuilder::new();
            assert!(b.set("TX_ID", "123", 0).is_ok());
            assert!(b.set("TX_TYPE", s, 0).is_ok());
            assert!(b.set("FROM_USER_ID", "0", 0).is_ok());
            assert!(b.set("TO_USER_ID", "456", 0).is_ok());
            assert!(b.set("AMOUNT", "100", 0).is_ok());
            assert!(b.set("TIMESTAMP", "1000", 0).is_ok());
            assert!(b.set("STATUS", "SUCCESS", 0).is_ok());
            assert!(b.set("DESCRIPTION", "\"test\"", 0).is_ok());

            let result = b.build(0);
            assert!(result.is_ok());

            if let Ok(tx) = result {
                assert_eq!(tx.tx_type, expected);
            }
        }
    }

    #[test]
    fn build_all_statuses() {
        for (s, expected) in [
            ("SUCCESS", TxStatus::Success),
            ("FAILURE", TxStatus::Failure),
            ("PENDING", TxStatus::Pending),
        ] {
            let mut b = TransactionBuilder::new();
            assert!(b.set("TX_ID", "123", 0).is_ok());
            assert!(b.set("TX_TYPE", "DEPOSIT", 0).is_ok());
            assert!(b.set("FROM_USER_ID", "0", 0).is_ok());
            assert!(b.set("TO_USER_ID", "456", 0).is_ok());
            assert!(b.set("AMOUNT", "100", 0).is_ok());
            assert!(b.set("TIMESTAMP", "1000", 0).is_ok());
            assert!(b.set("STATUS", s, 0).is_ok());
            assert!(b.set("DESCRIPTION", "\"test\"", 0).is_ok());

            let result = b.build(0);
            assert!(result.is_ok());

            if let Ok(tx) = result {
                assert_eq!(tx.status, expected);
            }
        }
    }

    #[test]
    fn set_duplicate_field_error() {
        let mut b = TransactionBuilder::new();
        assert!(b.set("TX_ID", "123", 0).is_ok());

        let err = b.set("TX_ID", "456", 5);
        assert!(err.is_err());

        if let Err(ReaderError::DuplicateField { line_no, field }) = err {
            assert_eq!(line_no, 5);
            assert_eq!(field, "TX_ID");
        } else {
            panic!("expected DuplicateField");
        }
    }

    #[test]
    fn set_duplicate_all_fields() {
        for field_name in schema::FIELDS_NAMES {
            let mut b = complete_builder();
            let err = b.set(field_name, "\"test\"", 10);

            assert!(
                matches!(&err, Err(ReaderError::DuplicateField { field, .. }) if field == field_name),
                "expected DuplicateField for {}, got {:?}",
                field_name,
                err
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Missing fields
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn build_empty_reports_all_missing() {
        let b = TransactionBuilder::new();
        let err = b.build(42);
        assert!(err.is_err());

        if let Err(ReaderError::MissingFields { line_no, fields }) = err {
            assert_eq!(line_no, 42);
            assert_eq!(fields.len(), 8);
            assert!(fields.contains(&"TX_ID".to_string()));
            assert!(fields.contains(&"DESCRIPTION".to_string()));
        } else {
            panic!("expected MissingFields");
        }
    }

    #[test]
    fn build_partial_reports_missing() {
        let mut b = TransactionBuilder::new();
        assert!(b.set("TX_ID", "123", 0).is_ok());
        assert!(b.set("TX_TYPE", "DEPOSIT", 0).is_ok());

        let err = b.build(5);
        assert!(err.is_err());

        if let Err(ReaderError::MissingFields { line_no, fields }) = err {
            assert_eq!(line_no, 5);
            assert_eq!(fields.len(), 6);
            assert!(!fields.contains(&"TX_ID".to_string()));
            assert!(!fields.contains(&"TX_TYPE".to_string()));
            assert!(fields.contains(&"AMOUNT".to_string()));
        } else {
            panic!("expected MissingFields");
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Unknown field
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn set_unknown_field_error() {
        let mut b = TransactionBuilder::new();
        let err = b.set("UNKNOWN_FIELD", "value", 10);
        assert!(err.is_err());

        if let Err(ReaderError::UnknownField { line_no, field }) = err {
            assert_eq!(line_no, 10);
            assert_eq!(field, "UNKNOWN_FIELD");
        } else {
            panic!("expected UnknownField");
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Invalid u64 values
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn set_invalid_u64_negative() {
        let mut b = TransactionBuilder::new();
        let err = b.set("TX_ID", "-1", 3);
        assert!(err.is_err());

        if let Err(ReaderError::InvalidFieldValue {
            line_no,
            field,
            value,
        }) = err
        {
            assert_eq!(line_no, 3);
            assert_eq!(field, "TX_ID");
            assert_eq!(value, "-1");
        } else {
            panic!("expected InvalidFieldValue");
        }
    }

    #[test]
    fn set_invalid_u64_overflow() {
        let mut b = TransactionBuilder::new();
        let err = b.set("AMOUNT", "18446744073709551616", 0);

        assert!(matches!(
            err,
            Err(ReaderError::InvalidFieldValue { field, .. }) if field == "AMOUNT"
        ));
    }

    #[test]
    fn set_invalid_u64_non_numeric() {
        let mut b = TransactionBuilder::new();
        let err = b.set("TIMESTAMP", "abc", 0);

        assert!(matches!(
            err,
            Err(ReaderError::InvalidFieldValue { field, .. }) if field == "TIMESTAMP"
        ));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Invalid TX_TYPE
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn set_invalid_tx_type() {
        let mut b = TransactionBuilder::new();
        let err = b.set("TX_TYPE", "INVALID", 7);
        assert!(err.is_err());

        if let Err(ReaderError::InvalidFieldValue {
            line_no,
            field,
            value,
        }) = err
        {
            assert_eq!(line_no, 7);
            assert_eq!(field, "TX_TYPE");
            assert_eq!(value, "INVALID");
        } else {
            panic!("expected InvalidFieldValue");
        }
    }

    #[test]
    fn set_tx_type_case_sensitive() {
        let mut b = TransactionBuilder::new();
        assert!(b.set("TX_TYPE", "Deposit", 0).is_err());

        let mut b2 = TransactionBuilder::new();
        assert!(b2.set("TX_TYPE", "deposit", 0).is_err());
    }

    // ─────────────────────────────────────────────────────────────────────
    // Invalid STATUS
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn set_invalid_status() {
        let mut b = TransactionBuilder::new();
        let err = b.set("STATUS", "UNKNOWN", 0);

        assert!(matches!(
            err,
            Err(ReaderError::InvalidFieldValue { field, .. }) if field == "STATUS"
        ));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Invalid DESCRIPTION (quotes)
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn set_description_valid_quotes() {
        for desc in ["\"hello\"", "\"\"", "\" \"", "\"a, b: c\""] {
            let mut b = TransactionBuilder::new();
            assert!(b.set("DESCRIPTION", desc, 0).is_ok());
        }
    }

    #[test]
    fn set_description_missing_quotes() {
        let mut b = TransactionBuilder::new();
        let err = b.set("DESCRIPTION", "no quotes", 0);

        assert!(matches!(
            &err,
            Err(ReaderError::InvalidRow { reason, .. }) if reason.contains("DESCRIPTION")
        ));
    }

    #[test]
    fn set_description_missing_opening_quote() {
        let mut b = TransactionBuilder::new();
        assert!(b.set("DESCRIPTION", "hello\"", 0).is_err());
    }

    #[test]
    fn set_description_missing_closing_quote() {
        let mut b = TransactionBuilder::new();
        assert!(b.set("DESCRIPTION", "\"hello", 0).is_err());
    }

    #[test]
    fn set_description_single_quote() {
        let mut b = TransactionBuilder::new();
        assert!(b.set("DESCRIPTION", "\"", 0).is_err());
    }

    // ─────────────────────────────────────────────────────────────────────
    // Line number propagation
    // ─────────────────────────────────────────────────────────────────────
    #[test]
    fn line_no_in_set_error() {
        let mut b = TransactionBuilder::new();
        let err = b.set("TX_ID", "bad", 123);

        if let Err(ReaderError::InvalidFieldValue { line_no, .. }) = err {
            assert_eq!(line_no, 123);
        } else {
            panic!("wrong error type");
        }
    }

    #[test]
    fn line_no_in_build_error() {
        let b = TransactionBuilder::new();
        let err = b.build(456);

        if let Err(ReaderError::MissingFields { line_no, .. }) = err {
            assert_eq!(line_no, 456);
        } else {
            panic!("wrong error type");
        }
    }
}
