const UNRESERVED_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.~";

pub struct UrlEncoding;

impl UrlEncoding {
    pub fn url_encode(input: String) -> String {
        let mut encoded = String::new();
        for ch in input.chars() {
            if UNRESERVED_CHARS.contains(ch) {
                encoded.push(ch);
            } else {
                encoded.push_str(&format!("%{:02X}", ch as u8));
            }
        }
        encoded
    }

    pub fn url_decode(input: String) -> Result<String, String> {
        let mut decoded = String::new();
        let mut chars = input.chars();

        while let Some(ch) = chars.next() {
            if ch == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        decoded.push(byte as char);
                    } else {
                        return Err(String::from("Invalid percent-encoded sequence"));
                    }
                } else {
                    return Err(String::from("Incomplete percent-encoded sequence"));
                }
            } else {
                decoded.push(ch);
            }
        }

        Ok(decoded)
    }
}