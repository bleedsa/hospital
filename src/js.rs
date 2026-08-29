const JS: &str = include_str!("../js/base.js");

pub fn base() -> String {
    JS.to_owned()
}
