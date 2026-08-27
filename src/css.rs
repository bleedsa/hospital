/** include the default css as a static str. RAWDOG. */
static DEFAULT_CSS: &str = include_str!("../css/default.css");

/** get the css stylesheet as a string to inject into <style> in page!{} */
pub fn css() -> String {
    DEFAULT_CSS.to_owned()
}
