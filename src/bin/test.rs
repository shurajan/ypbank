use std::io::{self, Cursor, Read};

fn main() -> io::Result<()> {
    let buffer = [
        0x59, 0x50, 0x42, 0x4e, // magic = "YPBN"
        0x00, 0x00, 0x00, 0x3f, // record_size = 63
        0x00, 0x03, 0x8d, 0x7e, 0xa4, 0xc6, 0x80, 0x00, // tx_id
        0x00, // tx_type
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // from_user_id
        0x00, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // to_user_id
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x64, // amount = 100
        0x00, 0x00, 0x01, 0x7c, 0x38, 0x94, 0xfa, 0x60, // timestamp
        0x01, // status
        0x00, 0x00, 0x00, 0x11, // desc_len = 17
        // description string (17 bytes)
        0x22, 0x52, 0x65, 0x63, 0x6f, 0x72, 0x64, 0x20, 0x6e, 0x75, 0x6d, 0x62, 0x65, 0x72, 0x20,
        0x31, 0x22,
    ];

    let mut cursor = Cursor::new(buffer);

    // ---- читаем magic ----
    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic)?;
    let magic_str = std::str::from_utf8(&magic).unwrap();

    // ---- record size ----
    let mut record_size_buf = [0u8; 4];
    cursor.read_exact(&mut record_size_buf)?;
    let record_size = u32::from_be_bytes(record_size_buf);

    // ---- tx_id ----
    let mut tx_id_buf = [0u8; 8];
    cursor.read_exact(&mut tx_id_buf)?;
    let tx_id = u64::from_be_bytes(tx_id_buf);

    // ---- tx_type ----
    let mut tx_type_buf = [0u8; 1];
    cursor.read_exact(&mut tx_type_buf)?;
    let tx_type = tx_type_buf[0];

    // ---- from_user_id ----
    let mut from_buf = [0u8; 8];
    cursor.read_exact(&mut from_buf)?;
    let from_user_id = u64::from_be_bytes(from_buf);

    // ---- to_user_id ----
    let mut to_buf = [0u8; 8];
    cursor.read_exact(&mut to_buf)?;
    let to_user_id = u64::from_be_bytes(to_buf);

    // ---- amount ----
    let mut amount_buf = [0u8; 8];
    cursor.read_exact(&mut amount_buf)?;
    let amount = u64::from_be_bytes(amount_buf);

    // ---- timestamp ----
    let mut ts_buf = [0u8; 8];
    cursor.read_exact(&mut ts_buf)?;
    let timestamp = u64::from_be_bytes(ts_buf);

    // ---- status ----
    let mut status_buf = [0u8; 1];
    cursor.read_exact(&mut status_buf)?;
    let status = status_buf[0];

    // ---- description length ----
    let mut desc_len_buf = [0u8; 4];
    cursor.read_exact(&mut desc_len_buf)?;
    let desc_len = u32::from_be_bytes(desc_len_buf);

    // ---- description ----
    let mut desc_bytes = vec![0u8; desc_len as usize];
    cursor.read_exact(&mut desc_bytes)?;
    let description = String::from_utf8_lossy(&desc_bytes);

    // ===============================
    // ✅ PRINT ALL FIELDS
    // ===============================

    println!("========== HEADER ==========");
    println!("Magic:        {}", magic_str);
    println!("Record size:  {}", record_size);

    println!("\n========== TRANSACTION ==========");
    println!("Tx ID:        {}", tx_id);
    println!("Tx Type:      {}", tx_type);
    println!("From User:    {}", from_user_id);
    println!("To User:      {}", to_user_id);
    println!("Amount:       {}", amount);
    println!("Timestamp:    {}", timestamp);
    println!("Status:       {}", status);

    println!("\n========== DESCRIPTION ==========");
    println!("Desc length:  {}", desc_len);
    println!("Desc text:    {}", description);

    Ok(())
}
