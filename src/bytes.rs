use std::cmp;

pub struct Bytes {
    bytes: f64,
}

impl Bytes {
    const UNIT_PREFIXES: [&'static str; 6] = ["", "k", "M", "G", "T", "P"];

    pub const fn new(bytes: u64) -> Self {
        Self {
            bytes: bytes as f64,
        }
    }
}

impl std::fmt::Display for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let num = self.bytes;
        debug_assert!(num >= 0.0);
        if num < 1_f64 {
            return write!(f, "{} B", num);
        }
        let delimiter = 1000_f64;
        let exponent = cmp::min((num.ln() / 6.90775).floor() as i32, 4);

        write!(f, "{:.2} ", num / delimiter.powi(exponent))?;
        write!(f, "{}B", Bytes::UNIT_PREFIXES[exponent as usize])
    }
}
