//! Writes the 4 MiB Salsa streams used by libsodium's core3 test.
//! Run through verify_salsa.py; this is an independent published hash check.
use std::io::Write;
use urng::{Rng, Salsa8, Salsa20};

fn main() -> std::io::Result<()> {
    let key = [
        0xdc, 0x90, 0x8d, 0xda, 0x0b, 0x93, 0x44, 0xa9, 0x53, 0x62, 0x9b, 0x73, 0x38, 0x20, 0x77,
        0x88, 0x80, 0xf3, 0xce, 0xb4, 0x21, 0xbb, 0x61, 0xb9, 0x1c, 0xbd, 0x4c, 0x3e, 0x66, 0x25,
        0x6c, 0xe4,
    ];
    let nonce = [0x82, 0x19, 0xe0, 0x03, 0x6b, 0x7a, 0x0b, 0x37];
    let mut out = std::io::BufWriter::new(std::io::stdout().lock());
    if std::env::args().nth(1).as_deref() == Some("8") {
        let mut rng = Salsa8::from_key_nonce(key, nonce, 0);
        for _ in 0..65536 * 16 {
            out.write_all(&rng.nextu().to_le_bytes())?;
        }
    } else {
        let mut rng = Salsa20::from_key_nonce(key, nonce, 0);
        for _ in 0..65536 * 16 {
            out.write_all(&rng.nextu().to_le_bytes())?;
        }
    }
    out.flush()
}
