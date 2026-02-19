# comparer

CLI tool for comparing transaction records from two files. Detects three kinds of differences: records present only in the left file, only in the right file, and records present in both but with mismatched fields.

## Build

```bash
cargo build --release
```

## Usage

```bash
comparer -l <FILE1> -L <FORMAT> -r <FILE2> -R <FORMAT>
```

### Arguments

| Flag | Description |
|------|-------------|
| `-l` | Path to the first file |
| `-L` | Format of the first file (`csv`, `txt`, `bin`) |
| `-r` | Path to the second file |
| `-R` | Format of the second file (`csv`, `txt`, `bin`) |

### Examples

```bash
# Compare two CSV files
comparer -l transactions_jan.csv -L csv -r transactions_feb.csv -R csv

# Compare a CSV file against a binary export
comparer -l export.csv -L csv -r backup.bin -R bin
```

## Output

```
[42] mismatched
  < tx_id=42 type=Transfer from=100 to=200 amount=5000 status=Success ts=1700000000000 desc=payment
  > tx_id=42 type=Transfer from=100 to=200 amount=9999 status=Failed  ts=1700000000000 desc=payment
    ~ amount:  5000 -> 9999
    ~ status:  Success -> Failed

[57] only in transactions_jan.csv
  < tx_id=57 type=Transfer from=300 to=400 amount=1500 status=Success ts=1700000001000 desc=refund

[99] only in transactions_feb.csv
  > tx_id=99 type=Deposit from=0 to=500 amount=200 status=Success ts=1700000002000 desc=deposit
```

Legend:
- `<` — record from the first file
- `>` — record from the second file
- `~` — specific fields that differ (only shown for `mismatched` records)

If both files contain identical records, a confirmation message is printed instead.
