use rusqlite::{Connection, Row, params};

use crate::{pre::*, passwd};
use std::{fs, path::Path};

fn row2user<'a>(r: &Row<'a>) -> rusqlite::Result<User> {
    let b: Option<String> = r.get(4)?;
    Ok(User {
        id: r.get(0)?,
        name: r.get(1)?,
        hash: r.get(2)?,
        admin: r.get(3)?,
        bio: if let Some(b) = b { b } else { String::new() },
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub hash: String,
    pub bio: String,
    pub admin: bool,
}

pub struct Db {
    pub sql: Connection,
}

impl Db {
    pub fn create(p: &str) -> R<Self> {
        let p = Path::new(p);

        /* if the run dir does not exist, we create it for you,
         * little special snowflake. */
        if let Some(d) = p.parent()
            && !un!(fs::exists(d))
        {
            un!(fs::create_dir_all(d));
        }

        /* open */
        let sql = un!(Connection::open(p));

        /* default table query */
        let q = r#"
        create table if not exists users (
            id integer primary key autoincrement,
            name text not null,
            hash text not null,
            admin boolean not null,
            bio text
        );
        "#;

        un!(sql.execute(q, []));
        Ok(Self { sql })
    }

    #[inline(always)]
    pub fn new() -> R<Self> {
        Self::create(&CFG.server.db)
    }

    pub fn new_user<N, P>(&self, n: N, p: P) -> R<User>
    where
        N: AsRef<str>,
        P: AsRef<str>,
    {
        /* as_ref() our params rq */
        let (n, p) = (n.as_ref(), p.as_ref());

        /* check to make sure the user doesn't already exist */
        let mut q =
            un!(self.sql.prepare("select id from users where name = ?1"));
        if let Ok(Some(_)) = un!(q.query(params![n])).next() {
            return err_fmt!("Db::new_user({n}): user already exists!");
        }

        /* make the hash */
        let h = passwd::hash(p)?;

        /* insert the user */
        un!(self.sql.execute(
            "
            insert into users (name, hash, admin)
            values (?1, ?2, 0);
            ",
            params![n, h]
        ));
    
        /* get the user back again */
        let r = un!(self.sql.prepare("select * from users where name = ?1"))
            .query_map(&[n], row2user)
            .map(|mut i| i.next());

        if let Some(u) = un!(r) {
            re!(u)
        } else {
            err_fmt!("Db::create({n}): user created, but not found")
        }
    }

    pub fn get_user(&self, id: i64) -> R<User> {
        let r = un!(self.sql.prepare("select * from users where id = ?1"))
            .query_map((id,), row2user)
            .map(|mut i| i.next());

        if let Some(u) = un!(r) {
            re!(u)
        } else {
            err_fmt!("Db::get_user({id}): user not found")
        }
    }

    pub fn get_user_by_name<N>(&self, n: N) -> R<User>
    where
        N: AsRef<str>,
    {
        let n = n.as_ref();
        let r = un!(self.sql.prepare("select * from users where name = ?1"))
            .query_map((n,), row2user)
            .map(|mut i| i.next());

        if let Some(u) = un!(r) {
            re!(u)
        } else {
            err_fmt!("Db::get_user_by_name({n}): user not found")
        }
    }

    #[inline(always)]
    pub fn new_admin(&self, id: i64) -> R<()> {
        re!(self.sql.execute(
            "
            update users
            set admin = 1
            where id = ?1
            ",
            params![id]
        )
            .map(|_| ()))
    }
}

#[cfg(test)]
mod test {
    use crate::{passwd, db::Db};
    use std::fs;

    fn O(p: &str) -> Db {
        if fs::exists(p).unwrap() {
            fs::remove_file(p).unwrap();
        }

        Db::create(p).unwrap()
    }

    #[test]
    fn empty_db() {
        let _ = O("run/test/empty.db");
    }

    #[test]
    fn new_user() {
        let db = O("run/test/new_user.db");
        let u = db.new_user("skylar", "empty").unwrap();

        assert_eq!(&u.name, "skylar");
        assert!(passwd::verify("empty", &u.hash).unwrap());
    }

    #[test]
    fn get_user() {
        let db = O("run/test/get_user.db");
        let u = db.new_user("skylar", "empty").unwrap();
        let u = db.get_user(u.id).unwrap();

        assert_eq!(&u.name, "skylar");
        assert!(passwd::verify("empty", &u.hash).unwrap());
    }

    #[test]
    fn get_user_by_name() {
        let db = O("run/test/get_user_by_name.db");
        let u = db.new_user("skylar", "empty").unwrap();
        let u = db.get_user_by_name(&u.name).unwrap();

        assert_eq!(&u.name, "skylar");
        assert!(passwd::verify("empty", &u.hash).unwrap());
    }

    #[test]
    fn mk_user_admin() {
        let db = O("run/test/mk_user_admin.db");
        let u = db.new_user("skylar", "empty").unwrap();

        assert_eq!(u.admin, false);

        db.new_admin(u.id).unwrap();

        let u = db.get_user(u.id).unwrap();
        assert_eq!(u.admin, true);
    }
}
