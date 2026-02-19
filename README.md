# ypbank

A Rust workspace for processing and managing financial transactions. Supports encoding/decoding in CSV, plain text, and binary formats.

## Workspace

| Crate | Type | Description |
|-------|------|-------------|
| `parser` | Library | Core codec library — encodes/decodes transactions across all formats |
| `converter` | CLI | Converts transaction files between formats |
| `comparer` | CLI | Compares two transaction files and reports differences |

## Transaction Fields

`tx_id`, `tx_type` (Deposit/Transfer/Withdrawal), `from_user_id`, `to_user_id`, `amount`, `timestamp`, `status` (Success/Failure/Pending), `description`

## Usage

### converter

```
converter -i <FILE> -f <FORMAT> -t <FORMAT>
```

| Flag | Description |
|------|-------------|
| `-i` | Input file path |
| `-f` | Input format (`csv`, `txt`, `bin`) |
| `-t` | Output format (`csv`, `txt`, `bin`) |

```bash
converter -i transactions.csv -f csv -t bin > transactions.bin
converter -i transactions.bin -f bin -t txt
```

### comparer

```
comparer -l <FILE1> -L <FORMAT> -r <FILE2> -R <FORMAT>
```

| Flag | Description |
|------|-------------|
| `-l` | Left file path |
| `-L` | Left file format (`csv`, `txt`, `bin`) |
| `-r` | Right file path |
| `-R` | Right file format (`csv`, `txt`, `bin`) |

```bash
comparer -l left.csv -L csv -r right.txt -R txt
comparer -l export.csv -L csv -r backup.bin -R bin
```

Output marks records only in left with `<`, only in right with `>`, and mismatched fields with `~`.

## Build

```bash
# Build all
cargo build --release

# Run tests
cargo test
```

Binaries will be at `target/release/converter` and `target/release/comparer`.
