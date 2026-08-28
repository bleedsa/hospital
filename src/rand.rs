use crate::pre::*;

/** make a random string `L` long */
pub fn rand_str<const L: usize>() -> R<String> {
    /* make an empty buffer */
    let mut buf = [0u8; L];

    /* fill the buf */
    for i in 0..L {
        buf[i] = rand::random_range(97..122);
    }

    /* convert */
    re!(str::from_utf8(&buf[..])).map(|s| s.to_string())
}

#[test]
fn mk_random_str() {
    let s = rand_str::<128>().unwrap();

    assert_eq!(s.len(), 128);
}
