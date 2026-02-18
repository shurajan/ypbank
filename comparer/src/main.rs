use anyhow::Context;
use clap::{Parser, ValueEnum};
use parser::{Bin, Csv, Decoder, Transaction, Txt};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "comparer",
    version = "0.1.0",
    about = "Compare transactions from two files and show the diff",
    long_about = None,
    arg_required_else_help = true
)]
struct Args {
    /// File1 path
    #[arg(short = 'l', long, value_name = "FILE")]
    file1: PathBuf,

    /// File1 format
    #[arg(short = 'L', long, value_name = "FORMAT")]
    format1: Format,

    /// File2 path
    #[arg(short = 'r', long, value_name = "FILE")]
    file2: PathBuf,

    /// File2 format
    #[arg(short = 'R', long, value_name = "FORMAT")]
    format2: Format,
}

#[derive(Debug, Clone, ValueEnum)]
enum Format {
    Csv,
    Txt,
    Bin,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let file1 = File::open(&args.file1)
        .with_context(|| format!("failed to open file: {}", args.file1.display()))?;

    let file2 = File::open(&args.file2)
        .with_context(|| format!("failed to open file: {}", args.file2.display()))?;

    let mut reader1 = BufReader::new(file1);
    let txs1 = decode(&args.format1, &mut reader1)?;

    let mut reader2 = BufReader::new(file2);
    let txs2 = decode(&args.format2, &mut reader2)?;

    let result = compare(&txs1, &txs2);

    for (key, diff) in result {
        match diff {
            Diff::OnlyLeft(r) => println!("[{key}] only in file 1 {r:?}"),
            Diff::OnlyRight(r) => println!("[{key}] only in file 2 {r:?}"),
            Diff::Mismatched(l, r) => println!("[{key}] mismatched {l:?} -> {r:?}"),
        }
    }

    Ok(())
}

fn decode(format: &Format, reader: &mut impl io::Read) -> anyhow::Result<Vec<Transaction>> {
    match format {
        Format::Csv => Csv.decode(reader),
        Format::Txt => Txt.decode(reader),
        Format::Bin => Bin.decode(reader),
    }
    .context("failed to decode transactions")
}

#[derive(Debug)]
enum Diff<'a> {
    OnlyLeft(&'a Transaction),
    OnlyRight(&'a Transaction),
    Mismatched(&'a Transaction, &'a Transaction),
}

fn compare<'a>(left: &'a [Transaction], right: &'a [Transaction]) -> Vec<(u64, Diff<'a>)> {
    let left_map: BTreeMap<u64, &Transaction> = left.iter().map(|tx| (tx.tx_id, tx)).collect();

    let right_map: BTreeMap<u64, &Transaction> = right.iter().map(|tx| (tx.tx_id, tx)).collect();

    let mut result = Vec::new();

    let mut l_iter = left_map.iter();
    let mut r_iter = right_map.iter();

    let mut l = l_iter.next();
    let mut r = r_iter.next();

    loop {
        match (l, r) {
            (None, None) => break,
            (Some((lk, lv)), Some((rk, rv))) => match lk.cmp(&rk) {
                Ordering::Less => {
                    result.push((*lk, Diff::OnlyLeft(lv)));
                    l = l_iter.next();
                }
                Ordering::Equal => {
                    if lv != rv {
                        result.push((*lk, Diff::Mismatched(lv, rv)));
                    }
                    l = l_iter.next();
                    r = r_iter.next();
                }
                Ordering::Greater => {
                    result.push((*rk, Diff::OnlyRight(rv)));
                    r = r_iter.next();
                }
            },
            (Some((lk, lv)), None) => {
                result.push((*lk, Diff::OnlyLeft(lv)));
                l = l_iter.next();
            }
            (None, Some((rk, rv))) => {
                result.push((*rk, Diff::OnlyRight(rv)));
                r = r_iter.next();
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::{TxStatus, TxType};

    fn tx(tx_id: u64, amount: u64, status: TxStatus) -> Transaction {
        Transaction {
            tx_id,
            tx_type: TxType::Transfer,
            from_user_id: 100,
            to_user_id: 200,
            amount,
            timestamp: 1700000000000,
            status,
            description: "test".to_string(),
        }
    }

    fn default_tx(tx_id: u64) -> Transaction {
        tx(tx_id, 1000, TxStatus::Success)
    }

    // --- normal ---

    #[test]
    fn test_both_empty() {
        assert!(compare(&[], &[]).is_empty());
    }

    #[test]
    fn test_identical() {
        let left = vec![default_tx(1), default_tx(2), default_tx(3)];
        let right = left.clone();
        assert!(compare(&left, &right).is_empty());
    }

    // --- only left ---

    #[test]
    fn test_all_only_left() {
        let left = vec![default_tx(1), default_tx(2)];
        let right = vec![];
        let result = compare(&left, &right);
        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], (1, Diff::OnlyLeft(_))));
        assert!(matches!(result[1], (2, Diff::OnlyLeft(_))));
    }

    #[test]
    fn test_only_left_value_correct() {
        let left = vec![default_tx(1)];
        let right = vec![];
        let result = compare(&left, &right);
        if let (1, Diff::OnlyLeft(l)) = &result[0] {
            assert_eq!(l.tx_id, 1);
        } else {
            panic!("expected OnlyLeft with tx_id=1");
        }
    }

    // --- only right ---

    #[test]
    fn test_all_only_right() {
        let left = vec![];
        let right = vec![default_tx(1), default_tx(2)];
        let result = compare(&left, &right);
        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], (1, Diff::OnlyRight(_))));
        assert!(matches!(result[1], (2, Diff::OnlyRight(_))));
    }

    #[test]
    fn test_only_right_value_correct() {
        let left = vec![default_tx(2)];
        let right = vec![default_tx(1), default_tx(2)];
        let result = compare(&left, &right);
        assert_eq!(result.len(), 1);
        if let (1, Diff::OnlyRight(r)) = &result[0] {
            assert_eq!(r.tx_id, 1);
        } else {
            panic!("expected OnlyRight with tx_id=1");
        }
    }

    // --- diff ---

    #[test]
    fn test_mismatched_amount() {
        let left = vec![tx(1, 100, TxStatus::Success)];
        let right = vec![tx(1, 999, TxStatus::Success)];
        let result = compare(&left, &right);
        assert_eq!(result.len(), 1);
        if let (1, Diff::Mismatched(l, r)) = &result[0] {
            assert_eq!(l.amount, 100);
            assert_eq!(r.amount, 999);
        } else {
            panic!("expected Mismatched");
        }
    }

    #[test]
    fn test_mismatched_status() {
        let left = vec![tx(1, 1000, TxStatus::Success)];
        let right = vec![tx(1, 1000, TxStatus::Failure)];
        let result = compare(&left, &right);
        assert_eq!(result.len(), 1);
        if let (1, Diff::Mismatched(l, r)) = &result[0] {
            assert_eq!(l.status, TxStatus::Success);
            assert_eq!(r.status, TxStatus::Failure);
        } else {
            panic!("expected Mismatched");
        }
    }

    // --- смешанный сценарий ---

    #[test]
    fn test_mixed() {
        let left = vec![default_tx(1), default_tx(2), tx(3, 300, TxStatus::Success)];
        let right = vec![default_tx(2), tx(3, 999, TxStatus::Success), default_tx(4)];

        let result = compare(&left, &right);
        assert_eq!(result.len(), 3);
        assert!(matches!(result[0], (1, Diff::OnlyLeft(_))));
        assert!(matches!(result[1], (3, Diff::Mismatched(_, _))));
        assert!(matches!(result[2], (4, Diff::OnlyRight(_))));
    }

    #[test]
    fn test_mixed_left_tail() {
        let left = vec![default_tx(1), default_tx(2), default_tx(3)];
        let right = vec![default_tx(1)];
        let result = compare(&left, &right);
        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], (2, Diff::OnlyLeft(_))));
        assert!(matches!(result[1], (3, Diff::OnlyLeft(_))));
    }

    #[test]
    fn test_mixed_right_tail() {
        let left = vec![default_tx(1)];
        let right = vec![default_tx(1), default_tx(2), default_tx(3)];
        let result = compare(&left, &right);
        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], (2, Diff::OnlyRight(_))));
        assert!(matches!(result[1], (3, Diff::OnlyRight(_))));
    }
}
