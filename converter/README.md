# converter

CLI tool for converting transaction files between formats. Reads an input file and writes the result to stdout in the specified output format.

## Build

```bash
cargo build --release
```

## Usage

```bash
converter -i <FILE> -f <FORMAT> -t <FORMAT>
```

### Arguments

| Flag | Description |
|------|-------------|
| `-i` | Path to the input file |
| `-f` | Input format (`csv`, `txt`, `bin`) |
| `-t` | Output format (`csv`, `txt`, `bin`) |

### Examples

```bash
# Convert CSV to binary
converter -i transactions.csv -f csv -t bin > transactions.bin

# Convert binary to TXT
converter -i transactions.bin -f bin -t txt

# Convert TXT to CSV and save to file
converter -i transactions.txt -f txt -t csv > transactions.csv
```

## Supported Formats

| Format | Description |
|--------|-------------|
| `csv`  | Comma-separated values with a header row |
| `txt`  | Human-readable plain text |
| `bin`  | Compact binary encoding |

## Notes

Output is written to **stdout**, so use shell redirection (`>`) to save it to a file. This makes the tool easy to compose with other CLI utilities via pipes.