//! The code the recovery's own lock screen is opened with. It is not the
//! phone's screen lock and asks nothing of the secure world: the panel
//! locks so that a finger brushing it in a bag opens nothing, and the code
//! is what a hand that meant to be there types.
//!
//! What is kept is a scrypt hash of the code, salted with what /dev/urandom
//! gives, in one line: the format's tag, the code's length, the salt and
//! the hash, as hex. The length is there so the lock screen can tell when
//! enough has been typed to check it, and nothing else about the code is.

use anyhow::{bail, Result};
use scrypt::{scrypt, Params};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

/// The shortest and the longest code the lock screen takes, in digits.
pub const MIN: usize = 4;
pub const MAX: usize = 8;

/// What the hash line starts with, so a later format can be told apart.
const TAG: &str = "s1";
const SALT_BYTES: usize = 16;
const HASH_BYTES: usize = 32;
/// The work: N = 2^13, r = 8, p = 1, which is some 8 MiB and a few tens of
/// milliseconds on the phone. More than enough to keep a hand guessing from
/// getting anywhere, and quick enough to check while the key is pressed.
const LOG_N: u8 = 13;

/// What a code may be: digits only, between [`MIN`] and [`MAX`] of them.
pub fn allowed(code: &str) -> bool {
    (MIN..=MAX).contains(&code.chars().count()) && code.chars().all(|c| c.is_ascii_digit())
}

/// The line to keep for `code`, or why it is not one to keep.
pub fn hash(code: &str) -> Result<String> {
    if !allowed(code) {
        bail!("a code is {MIN} to {MAX} digits");
    }
    let salt = salt();
    let mut hash = [0u8; HASH_BYTES];
    let params = Params::new(LOG_N, 8, 1, HASH_BYTES).map_err(|e| anyhow::anyhow!("the code's hash settings: {e}"))?;
    scrypt(code.as_bytes(), &salt, &params, &mut hash).map_err(|e| anyhow::anyhow!("hash the code: {e}"))?;
    Ok(format!("{TAG}${}${}${}", code.chars().count(), hex(&salt), hex(&hash)))
}

/// Whether `code` is the one `recorded` was made from. A line that does not
/// read as one of these, or a code the lock screen would never take, is
/// false: there is nothing to open.
pub fn check(recorded: &str, code: &str) -> bool {
    let Some((salt, wanted, length)) = read(recorded) else { return false };
    if !allowed(code) || code.chars().count() != length {
        return false;
    }
    let Ok(params) = Params::new(LOG_N, 8, 1, HASH_BYTES) else { return false };
    let mut hash = [0u8; HASH_BYTES];
    if scrypt(code.as_bytes(), &salt, &params, &mut hash).is_err() {
        return false;
    }
    // The whole of both, whatever the first byte that differs.
    let mut differs = 0u8;
    for (a, b) in hash.iter().zip(wanted.iter()) {
        differs |= a ^ b;
    }
    differs == 0
}

/// How many digits the code behind `recorded` has, where the line reads as
/// one of these.
pub fn length(recorded: &str) -> Option<usize> {
    read(recorded).map(|(_, _, length)| length)
}

/// The salt, the hash and the code's length of a line, where the whole of
/// it reads as one of these.
fn read(recorded: &str) -> Option<([u8; SALT_BYTES], [u8; HASH_BYTES], usize)> {
    let mut parts = recorded.split('$');
    if parts.next() != Some(TAG) {
        return None;
    }
    let length: usize = parts.next()?.parse().ok()?;
    if !(MIN..=MAX).contains(&length) {
        return None;
    }
    let salt: [u8; SALT_BYTES] = unhex(parts.next()?)?.try_into().ok()?;
    let hash: [u8; HASH_BYTES] = unhex(parts.next()?)?.try_into().ok()?;
    parts.next().is_none().then_some((salt, hash, length))
}

/// Sixteen bytes to salt the hash with: what the kernel gives, and where
/// there is no such file, the clock mixed with this process and a counter.
fn salt() -> [u8; SALT_BYTES] {
    let mut salt = [0u8; SALT_BYTES];
    if let Ok(mut random) = File::open("/dev/urandom") {
        if random.read_exact(&mut salt).is_ok() {
            return salt;
        }
    }
    use std::sync::atomic::{AtomicU64, Ordering};
    static TURNS: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos());
    let turn = TURNS.fetch_add(1, Ordering::Relaxed);
    let mut hasher = Sha256::new();
    hasher.update(nanos.to_le_bytes());
    hasher.update(std::process::id().to_le_bytes());
    hasher.update(turn.to_le_bytes());
    salt.copy_from_slice(&hasher.finalize()[..SALT_BYTES]);
    salt
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(char::from_digit((b >> 4) as u32, 16).expect("a nibble is a hex digit"));
        out.push(char::from_digit((b & 0xf) as u32, 16).expect("a nibble is a hex digit"));
    }
    out
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    let bytes = text.as_bytes();
    (0..bytes.len() / 2)
        .map(|i| {
            let high = (bytes[i * 2] as char).to_digit(16)?;
            let low = (bytes[i * 2 + 1] as char).to_digit(16)?;
            Some((high * 16 + low) as u8)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hashed_code_is_the_one_the_hash_was_made_from() {
        let recorded = hash("513920").unwrap();
        assert!(check(&recorded, "513920"));
        assert!(!check(&recorded, "513921"));
        assert!(!check(&recorded, "51392"), "a shorter code is not tried at all");
        assert_eq!(length(&recorded), Some(6));
    }

    #[test]
    fn the_same_code_hashes_differently_each_time() {
        let (one, two) = (hash("1234").unwrap(), hash("1234").unwrap());
        assert_ne!(one, two, "the salt is not the same");
        assert!(check(&one, "1234") && check(&two, "1234"));
    }

    #[test]
    fn nothing_opens_a_line_that_does_not_read() {
        for bad in ["", "1234", "s1$6$zz$zz", "s2$6$0011$2233", "s1$6$0011", "s1$9$0011$2233"] {
            assert!(!check(bad, "513920"), "{bad:?} opens nothing");
            assert_eq!(length(bad), None, "{bad:?} names no length");
        }
    }

    #[test]
    fn a_code_is_four_to_eight_digits() {
        for code in ["123", "123456789", "12a4", "12 4", ""] {
            assert!(hash(code).is_err(), "{code:?} is not a code");
            assert!(!allowed(code));
        }
        for code in ["1234", "12345678"] {
            assert!(hash(code).is_ok(), "{code:?} is a code");
        }
    }
}
