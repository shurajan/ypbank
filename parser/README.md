# ypbank

Ypbank `parser` is a small Rust library for encoding and decoding financial transactions
in multiple formats.

It provides a unified API through the `Decoder` and `Encoder` traits, with
built-in support for:

- **CSV** (`Csv`)
- **Plain text** (`Txt`)
- **Binary** (`Bin`)

## Formats

- [CSV Specification](docs/YPBankCsvFormat_ru.md)
- [TXT Specification](docs/YPBankTextFormat_ru.md)
- [Binary Specification](docs/YPBankBinFormat_ru.md)

---

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
ypbank = "0.1"
```

---


## Usage

### Decode CSV

```rust
use std::io::Cursor;
use ypbank::{Decoder, Csv};

let data = r#"TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION
1001,DEPOSIT,0,501,50000,1672531200000,SUCCESS,"Initial funding"
"#;

let mut cursor = Cursor::new(data);
let txs = Csv.decode(&mut cursor).unwrap();

assert_eq!(txs.len(), 1);
```

---

### Round-trip Encoding/Decoding

```rust
use std::io::Cursor;
use ypbank::{Decoder, Encoder, Txt, Transaction, TxType, TxStatus};

let tx = Transaction {
    tx_id: 1001,
    tx_type: TxType::Deposit,
    from_user_id: 0,
    to_user_id: 501,
    amount: 50_000,
    timestamp: 1672531200000,
    status: TxStatus::Success,
    description: "\"Initial funding\"".to_string(),
};

let mut buf = Cursor::new(Vec::new());

Txt.encode(&[tx.clone()], &mut buf).unwrap();

buf.set_position(0);
let decoded = Txt.decode(&mut buf).unwrap();

assert_eq!(decoded[0], tx);
```

---

## Error Handling

Decoding returns `ReaderError`, encoding returns `WriterError`:

```rust
use ypbank::{Decoder, Csv};

let result = Csv.decode(&mut "invalid".as_bytes());

assert!(result.is_err());
```

---

## License

MIT
