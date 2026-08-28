use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

use crate::pre::*;

#[inline(always)]
pub fn hash<P>(p: P) -> R<String>
where
    P: AsRef<str>,
{
    let p = p.as_ref();

    /* make the rng and whatever */
    let ar = Argon2::default();

    Ok(un!(ar.hash_password(p.as_bytes())).to_string())
}

#[inline(always)]
pub fn verify<P, H>(p: P, h: H) -> R<bool>
where
    H: AsRef<str>,
    P: AsRef<str>,
{
    let parsed = un!(PasswordHash::new(h.as_ref()));
    re!(Argon2::default()
        .verify_password(p.as_ref().as_bytes(), &parsed)
        .map(|_| true))
}

#[test]
fn verify_a_passwd() {
    let p = "test123";
    let h = hash(p).unwrap();

    assert!(verify(p, h).unwrap());
}
