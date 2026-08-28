use crate::{db::Db, pre::*};
use axum::{extract::Path, response::Html};

pub async fn user(Path(id): Path<i64>) -> H<Html<String>> {
    let db = un!(Db::new());
    let u = un!(db.get_user(id));

    Ok(page!(db, {
        ("user {id}"),
        r#"
        <h1>user {id}</h1>
        <table>
            <tr>
                <td>name</td>
                <td>{name}</td>
            </tr>
            <tr>
                <td>hash</td>
                <td>{hash}</td>
            </tr>
            <tr>
                <td>bio</td>
                <td>{bio}</td>
            </tr>
            <tr>
                <td>admin?</td>
                <td>{admin}</td>
            </tr>
        </table>
        "#,
        name = u.name,
        hash = u.hash,
        bio = u.bio,
        admin = u.admin,
    }))
}
