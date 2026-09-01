use axum::{extract::Path, response::Html};
use axum_cookie::prelude::*;

use crate::{db::Db, pre::*};

pub async fn by_name(
    C: CookieManager,
    Path(n): Path<String>,
) -> H<Html<String>> {
    let db = Db::new()?;
    let me = db.me(&C)?;
    let u = db.get_user_by_name(&n)?;

    Ok(page!(db, Some(me.id), {
        ("~{}", h!(u.name)),
        r#"
        <h1>~{name} {admin}</h1>
        <p>{bio}</p>

        <h3>update user info</h3>
        <form method="post" action="/act/update-user">
            {update}
        </form>

        <h3>update user css</h3>
        <form method="post" action="/act/update-css">
            {css}
        </form>
        "#,
        admin = if u.admin {
            "(admin)"
        } else {
            ""
        },
        name = h!(u.name),
        bio = h!(u.bio),
        /* if the user is me, show a form to update user information */
        update = if me.id == u.id {
            format!(
                r#"
                <table>
                    <tr>
                        <td>bio</td>
                        <td><textarea name="bio" rows="3" cols="30"></textarea></td>
                    </tr>
                    <tr>
                        <td>password</td>
                        <td><input type="password" name="pass"></td>
                    </tr>
                    <tr>
                        <td>theme</td>
                        <td>
                            <select name="theme">{themes}</select>
                        </td>
                    </tr>
                </table>
                <input type="hidden" name="user" value="{id}">
                <input type="submit" value="go">
                "#,
                id = me.id,
                themes = css::get_theme_names()
                    .map(|n| format!(
                        r#"
                        <option value="{n}">{n}</option>
                        "#
                    ))
                    .collect::<String>(),
            )
        } else {
            String::new()
        },
        css = if me.id == u.id {
            let css = db.get_css(me.id)?.map(|c| (c.vars, c.css));

            format!(
                r#"
                <table>
                    <tr>
                        <td>css vars</td>
                        <td>
                            <textarea name="vars" rows="3" cols="30">{vars}</textarea>
                        </td>
                    </tr>
                    <tr>
                        <td>css body</td>
                        <td>
                            <textarea name="body" rows="3" cols="30">{body}</textarea>
                        </td>
                    </tr>
                </table>
                <input type="hidden" name="user" value="{id}">
                <input type="submit" value="go">
                "#,
                id = me.id,
                vars = css.as_ref().map(|(x, _)| x).unwrap_or(&String::new()),
                body = css.as_ref().map(|(_, x)| x).unwrap_or(&String::new()),
            )
        } else {
            String::new()
        },
    }))
}
