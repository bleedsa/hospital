use axum::{response::Html, extract::Path};
use crate::{pre::*, db::Db};

pub async fn user(Path(id): Path<i64>) -> H<Html<String>> {
    let db = un!(Db::new(), "/login");
    let u = un!(db.get_user(id), "/login");

    Ok(page! {
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
    })
}
