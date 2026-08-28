use rusqlite::{Connection, Row, params};
use axum_cookie::prelude::*;

use crate::{passwd, pre::*, rand::rand_str};
use std::{fs, path::Path};

pub const SESSION_HASH_LEN: usize = 64;

/** a user entry in the database */
#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub hash: String,
    pub bio: String,
    pub admin: bool,
}

/** convert a `Row` to a `User` */
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

/** repr for a session entry */
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub id: i64,
    pub hash: String,
    pub user: i64,
}

impl Session {
    #[inline(always)]
    pub fn new<'a>(r: &Row<'a>) -> rusqlite::Result<Self> {
        Ok(Session {
            id: r.get(0)?,
            hash: r.get(1)?,
            user: r.get(2)?,
        })
    }
}

/** repr for a board entry in the database */
#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    pub id: i64,
    pub name: String,
    pub desc: String,
}

impl Board {
    #[inline(always)]
    pub fn new<'a>(r: &Row<'a>) -> rusqlite::Result<Self> {
        Ok(Board {
            id: r.get(0)?,
            name: r.get(1)?,
            desc: r.get(2)?,
        })
    }
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
        Ok(Self { sql })
    }

    #[inline(always)]
    pub fn new() -> R<Self> {
        Self::create(&CFG.server.db)
    }

    /** initialize a database with the tables we need */
    pub fn init(self) -> R<Self> {
        /* create the init table if it doesn't exist */
        un!(self.sql.execute(
            r#"
            create table if not exists init (
                id integer primary key autoincrement
            );
            "#,
            []
        ));

        /* check the table to see if we already init. saves on a few queries */
        let q = un!(self.sql.prepare("select * from init"))
            .query_map((), |_| Ok(()))
            .map(|mut i| i.next());

        /* return if we found a row in the init table */
        if let Some(_) = un!(q) {
            return Ok(self);
        }

        /* default table querys */
        for q in [
            r#"
            create table if not exists users (
                id integer primary key autoincrement,
                name text not null,
                hash text not null,
                admin boolean not null,
                bio text
            );
            "#,
            r#"
            create table if not exists sessions (
                id integer primary key autoincrement,
                hash text not null,
                user integer not null
            );
            "#,
            r#"
            create table if not exists boards (
                id integer primary key autoincrement,
                name text not null,
                desc text not null
            );
            "#,
        ]
        .into_iter()
        {
            un!(self.sql.execute(q, []));
        }

        /* mark that we have, in fact, init */
        un!(self.sql.execute(
            "
            insert into init default values; 
            ",
            ()
        ));

        Ok(self)
    }

    /** get the current user based on the cookies */
    pub fn me(&self, c: &CookieManager) -> R<User> {
        let hash = if let Some(cookie) = c.get("session") {
            cookie.value().to_string()
        } else {
            return err_fmt!("Db::me(): not logged in");
        };

        let sesh_r = un!(self.sql.prepare(
            "
            select * from sessions
            where hash = ?1
            "
        ))
            .query_map((hash,), Session::new)
            .map(|mut i| i.next());

        let s = if let Some(s) = un!(sesh_r) {
            un!(s)
        } else {
            err_fmt!("Db::me(): no sessions found for user")?
        };

        self.get_user(s.user)
    }

    /** make a new user with name `n` and password `p` */
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

    /** get a user by id */
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

    /** get a user by name */
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

    /** make a user admin by id */
    #[inline(always)]
    pub fn new_admin(&self, id: i64) -> R<()> {
        re!(self
            .sql
            .execute(
                "
                update users
                set admin = 1
                where id = ?1
                ",
                params![id]
            )
            .map(|_| ()))
    }

    /** remove a user by id */
    #[inline(always)]
    pub fn rm_user(&self, id: i64) -> R<()> {
        re!(self
            .sql
            .execute(
                "
                delete from users
                where id = ?1
                ",
                params![id]
            )
            .map(|_| ()))
    }

    /** make a new session for a user */
    pub fn new_session(&self, id: i64) -> R<Session> {
        un!(self.sql.execute(
            "
            delete from sessions
            where id = ?1
            ",
            (id,)
        ));

        /* make a random hash string */
        let hash = rand_str::<SESSION_HASH_LEN>()?;

        un!(self.sql.execute(
            "
            insert into sessions (hash, user)
            values (?1, ?2)
            ",
            (hash, id),
        ));

        /* get the session & return */
        let r = un!(self.sql.prepare("select * from sessions where user = ?1"))
            .query_map((id,), Session::new)
            .map(|mut i| i.next());

        if let Some(s) = un!(r) {
            re!(s)
        } else {
            err_fmt!("Db::new_session({id}): session created, but not found")
        }
    }

    /** get a session for a user */
    pub fn get_session(&self, id: i64) -> R<Session> {
        let r = un!(self.sql.prepare("select * from sessions where user = ?1"))
            .query_map((id,), Session::new)
            .map(|mut i| i.next());

        if let Some(s) = un!(r) {
            re!(s)
        } else {
            err_fmt!("Db::get_session({id}): session not found")
        }
    }

    /** get a board from the database by name */
    pub fn get_board_by_name<N>(&self, n: N) -> R<Board>
    where
        N: AsRef<str>,
    {
        let n = n.as_ref();
        let r = un!(self.sql.prepare("select * from boards where name = ?1"))
            .query_map((n,), Board::new)
            .map(|mut i| i.next());

        if let Some(b) = un!(r) {
            re!(b)
        } else {
            err_fmt!("Db::get_board_by_name({n}): board not found")
        }
    }

    /** get a board by id */
    pub fn get_board(&self, id: i64) -> R<Board> {
        let r = un!(self.sql.prepare("select * from boards where id = ?1"))
            .query_map((id,), Board::new)
            .map(|mut i| i.next());

        if let Some(b) = un!(r) {
            re!(b)
        } else {
            err_fmt!("Db::get_board({id}): board not found")
        }
    }

    /** get an iterator of all boards */
    pub fn get_boards<'a>(&'a self) -> R<Vec<Board>> {
        let mut stm = un!(self.sql.prepare("select * from boards"));

        let map = stm
            .query_map((), Board::new)
            .map_err(|e| format!("Db::get_boards(): {e}"))?;

        map.map(|x| x.map_err(|e| format!("{e}"))).collect()
    }

    /** create a new board */
    pub fn new_board<N, D>(&self, n: N, d: D) -> R<Board>
    where
        N: AsRef<str>,
        D: AsRef<str>,
    {
        /* shadow generics */
        let (n, d) = (n.as_ref(), d.as_ref());

        /* check if this board already exists */
        if let r @ Ok(_) = self.get_board_by_name(n) {
            return r;
        }

        /* perform the insertion */
        un!(self.sql.execute(
            "
            insert into boards (name, desc)
            values (?1, ?2)
            ",
            (n, d)
        ));

        /* get the new board */
        match self.get_board_by_name(n) {
            r @ Ok(_) => r,
            Err(e) => err_fmt!("Db::new_board('{n}', '{d}'): {e}"),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        db::{Db, SESSION_HASH_LEN},
        passwd,
    };
    use std::{fs, iter::zip};

    fn O(p: &str) -> Db {
        if fs::exists(p).unwrap() {
            fs::remove_file(p).unwrap();
        }

        Db::create(p).unwrap().init().unwrap()
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

    #[test]
    fn rm_user() {
        let db = O("run/test/rm_user.db");

        let u = db.new_user("skylar", "empty").unwrap();
        assert_eq!("skylar", &u.name);

        db.rm_user(u.id).unwrap();
        assert!(db.get_user(u.id).is_err());
    }

    #[test]
    fn mk_session() {
        let db = O("run/test/mk_session.db");

        let u = db.new_user("skylar", "empty").unwrap();
        assert_eq!("skylar", &u.name);

        let s = db.new_session(u.id).unwrap();
        assert_eq!(s.user, u.id);
        assert_eq!(s.hash.len(), SESSION_HASH_LEN);
    }

    #[test]
    fn get_session() {
        let db = O("run/test/get_session.db");
        let u = db.new_user("skylar", "empty").unwrap();
        let s = db.new_session(u.id).unwrap();
        let s = db.get_session(s.user).unwrap();

        assert_eq!(s.user, u.id);
        assert_eq!(s.hash.len(), SESSION_HASH_LEN);
    }

    #[test]
    fn mk_board() {
        let db = O("run/test/mk_board.db");
        let b1 = db.new_board("general", "empty").unwrap();
        let b2 = db.get_board(b1.id).unwrap();
        assert_eq!(&b1, &b2);
    }

    #[test]
    fn get_boards() {
        let db = O("run/test/get_boards.db");

        let B = [("G", "g"), ("F", "f"), ("Abc", "Def")];
        let mut Xs = B
            .iter()
            .map(|(n, d)| db.new_board(n, d).unwrap().id)
            .collect::<Vec<_>>();
        let mut Ys = db
            .get_boards()
            .unwrap()
            .into_iter()
            .map(|b| b.id)
            .collect::<Vec<_>>();

        Xs.sort();
        Ys.sort();

        for (x, y) in zip(Xs, Ys) {
            assert_eq!(x, y);
        }
    }
}
