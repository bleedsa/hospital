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

    Ok(page!(db, {
        ("{}", h!(u.name)),
        r#"
        <h1>{name} {admin}</h1>
        <p>{bio}</p>
        <form method="post" action="/act/update-user">
            {update}
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
                </table>
                <input type="hidden" name="user" value="{id}">
                <input type="submit" value="go">
                "#,
                id = me.id,
            )
        } else {
            String::new()
        },
    }))
}
