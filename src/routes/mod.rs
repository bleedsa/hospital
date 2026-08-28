use crate::{pre::*, tags::tag};
use axum::response::Html;

pub mod act;
pub mod dbg;

/** index/homepage (GET /) */
pub async fn index() -> Html<String> {
    page! {
        ("index"),
        r#"
        <h1>{title}.</h1>
        <p>{tag}.</p>
        <hr>
        "#,
        title = (&*CFG).title(),
        tag = tag(),
    }
}

pub async fn login() -> Html<String> {
    page! {
        ("login"),
        r#"
        <h1>login.</h1>
        <div class="form">
            <form action="/act/login" method="post">
                <table>
                    <tr>
                        <td><span class="form-item">username</span></td>
                        <td>
                            <span class="form-item">
                                <input type="text" name="user">
                            </span>
                        </td>
                    </tr>
                    <tr>
                        <td><span class="form-item">password</span></td>
                        <td>
                            <span class="form-item">
                                <input type="password" name="pass">
                            </span>
                        </td>
                    </tr>
                </table>
                <input type="submit" value="go">
            </form>
        </div>
        "#
    }
}
