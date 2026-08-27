use axum::response::Html;
use crate::pre::*;

/* a collection of taglines for the index page */
static TAGS: &[&str] = &[
    "haven for the huddled masses",
    "padded cell for autistic transexuals",
    "im whatever you want me to be baby",
    "who are you, man?",
    "what lol",
    "SHUT THE FUCK UP",
    "powered by manic psychosis",
    "powered by delusions of grandeur",
    "powered by a 2015 DELL PC",
    "powered by illumos",
    "powered by monster energy and cigarettes",
    "you didn't see shit",
    "this website is a product of your imagination",
    "you don't know me",
    "whatever hoe",
    "insane asylum",
];

/* get a random tag out of TAGS */
#[inline(always)]
fn tag() -> &'static str {
    let L = TAGS.len() as u8; 
    let i = rand::random::<u8>() % L;
    TAGS[i as usize]
}

/* index/homepage (GET /) */
pub async fn index() -> Html<String> {
    page! {
        ("index"),
        r#"
        <h1>badboy hospital</h1>
        <p>{tag}.</p>
        <hr>
        "#,
        tag = tag(),
    }
}
