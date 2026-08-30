use axum::body::Bytes;
use axum_cookie::prelude::*;
use image::ImageReader;
use rusqlite::{Connection, Row, params};

use crate::{passwd, pre::*, rand::rand_str, css::{self, THEMES}};
use std::{fs, io::Cursor, path::Path};

pub const SESSION_HASH_LEN: usize = 512;

/** a user entry in the database */
#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub hash: String,
    pub bio: String,
    pub admin: bool,
    pub theme: Option<String>,
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
        theme: r.get(5)?,
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
    pub hidden: bool,
}

impl Board {
    #[inline(always)]
    pub fn new<'a>(r: &Row<'a>) -> rusqlite::Result<Self> {
        Ok(Board {
            id: r.get(0)?,
            name: r.get(1)?,
            desc: r.get(2)?,
            hidden: r.get(3)?,
        })
    }
}

pub struct File {
    pub id: i64,
    pub bytes: Vec<u8>,
}

impl File {
    #[inline(always)]
    pub fn new<'a>(r: &Row<'a>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: r.get(0)?,
            bytes: r.get(1)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Thread {
    pub id: i64,
    pub name: String,
    pub cont: String,
    pub hidden: bool,
    pub file: Option<i64>,
    pub board: i64,
    pub time: i64,
    pub author: i64,
    pub locked: Option<bool>,
}

impl Thread {
    #[inline(always)]
    pub fn new<'a>(r: &Row<'a>) -> rusqlite::Result<Self> {
        Ok(Thread {
            id: r.get(0)?,
            name: r.get(1)?,
            cont: r.get(2)?,
            hidden: r.get(3)?,
            file: r.get(4)?,
            board: r.get(5)?,
            time: r.get(6)?,
            author: r.get(7)?,
            locked: r.get(8)?,
        })
    }

    /** is this thread locked?
     * wrapped because `Thread::locked` is an `Option` for compat with
     * the old db */
    #[inline(always)]
    pub fn locked(&self) -> bool {
        if let Some(b) = self.locked { b } else { false }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Post {
    pub id: i64,
    pub cont: String,
    pub hidden: bool,
    pub file: Option<i64>,
    pub board: i64,
    pub thread: i64,
    pub time: i64,
    pub author: i64,
}

impl Post {
    #[inline(always)]
    pub fn new<'a>(r: &Row<'a>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: r.get(0)?,
            cont: r.get(1)?,
            hidden: r.get(2)?,
            file: r.get(3)?,
            board: r.get(4)?,
            thread: r.get(5)?,
            time: r.get(6)?,
            author: r.get(7)?,
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
        /* default table querys */
        for q in [
            r#"
            create table if not exists users (
                id integer primary key autoincrement,
                name text not null,
                hash text not null,
                admin boolean not null,
                bio text,
                theme text
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
                desc text not null,
                hidden boolean not null
            );
            "#,
            r#"
            create table if not exists threads (
                id integer primary key autoincrement,
                name text not null,
                cont text not null,
                hidden boolean not null,
                file integer,
                board integer not null,
                time integer not null,
                author integer not null,
                locked boolean
            );
            "#,
            r#"
            create table if not exists files (
                id integer primary key autoincrement,
                bytes blob not null
            );
            "#,
            r#"
            create table if not exists posts (
                id integer primary key autoincrement,
                cont text not null,
                hidden boolean not null,
                file integer,
                board integer not null,
                thread integer not null,
                time integer not null,
                author integer not null
            );
            "#,
        ]
        .into_iter()
        {
            un!(self.sql.execute(q, []));
        }

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
        /* make a random hash string */
        let hash = rand_str::<SESSION_HASH_LEN>()?;

        L!(("{} created new session {hash}", self.get_user(id)?.name) => {
            un!(self.sql.execute(
                "
                insert into sessions (hash, user)
                values (?1, ?2)
                ",
                (hash, id),
            ));
        });

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

    /** simple subroutine to iterate visible boards */
    #[inline(always)]
    pub fn get_visible_boards(&self) -> R<Vec<Board>> {
        let mut stm = un!(self.sql.prepare(
            "
            select * from boards
            where not hidden
            "
        ));

        let map = un!(stm.query_map((), Board::new));
        map.map(|x| re!(x)).collect()
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
            insert into boards (name, desc, hidden)
            values (?1, ?2, false)
            ",
            (n, d)
        ));

        /* get the new board */
        match self.get_board_by_name(n) {
            r @ Ok(_) => r,
            Err(e) => err_fmt!("Db::new_board('{n}', '{d}'): {e}"),
        }
    }

    /** hide a board from view */
    #[inline(always)]
    pub fn hide_board(&self, id: i64) -> R<Board> {
        /* verify that the board actually exists */
        /* TODO: does retrieving the board twice make this too slow? */
        let _ = self.get_board(id)?;

        un!(self.sql.execute(
            "
            update boards
            set hidden = true
            where id = ?1
            ",
            (id,)
        ));

        self.get_board(id)
    }

    /** make a board visible */
    pub fn visible_board(&self, id: i64) -> R<Board> {
        let _ = self.get_board(id)?;

        un!(self.sql.execute(
            "
            update boards
            set hidden = false
            where id = ?1
            ",
            (id,)
        ));

        self.get_board(id)
    }

    /** add a new file to the db */
    pub fn new_file(&self, file: Bytes) -> R<File> {
        let vec = file.to_vec();

        /* verify that the file is, in fact, an image */
        let im = un!(ImageReader::new(Cursor::new(&vec)).with_guessed_format());
        let _ = un!(im.decode());

        L!(("creating new file") => {
            un!(self.sql.execute(
                "insert into files (bytes)
                values (?1)
               ",
                (vec,)
            ));
        });

        let r = un!(self.sql.prepare(
            "
            select * from files
            where id = LAST_INSERT_ROWID()
            ",
        ))
        .query_map((), File::new)
        .map(|mut i| i.next());

        if let Some(f) = un!(r) {
            re!(f)
        } else {
            err_fmt!("Db::new_file(): file created, but not found")
        }
    }

    pub fn get_file(&self, id: i64) -> R<File> {
        let r = un!(self.sql.prepare(
            "
            select * from files
            where id = ?1
            ",
        ))
        .query_map((id,), File::new)
        .map(|mut i| i.next());

        if let Some(f) = un!(r) {
            re!(f)
        } else {
            err_fmt!("Db::get_file({id}): file not found")
        }
    }

    /** get a thread by id */
    pub fn get_thread(&self, id: i64) -> R<Thread> {
        let r = un!(self.sql.prepare(
            "
            select * from threads
            where id = ?1
            "
        ))
        .query_map((id,), Thread::new)
        .map(|mut i| i.next());

        if let Some(t) = un!(r) {
            re!(t)
        } else {
            err_fmt!("Db::get_thread({id}): thread not found")
        }
    }

    pub fn get_threads(&self, board: i64) -> R<Vec<Thread>> {
        let mut r = un!(self.sql.prepare(
            "
            select * from threads
            where board = ?1
            "
        ));

        let map = un!(r.query_map((board,), Thread::new));
        let vec = map.map(|x| re!(x)).collect::<R<Vec<_>>>()?;

        /* sort by which thread has the most recent post */
        let mut zipped = Vec::new();
        for t in vec.iter() {
            let mut times = self
                .get_posts(t.id)?
                .into_iter()
                .map(|p| p.time)
                .collect::<Vec<_>>();
            times.sort();
            /* safety */
            if let Some(e) = times.last() {
                zipped.push((*e, t.clone()));
            } else {
                zipped.push((t.time, t.clone()));
            }
        }

        /* sort by time */
        zipped.sort_by_key(|(t, _)| *t);
        Ok(zipped.into_iter().rev().map(|(_, x)| x).collect())
    }

    pub fn get_visible_threads(
        &self,
        board: i64,
    ) -> R<impl Iterator<Item = Thread>> {
        let ts = self.get_threads(board)?;
        Ok(ts.into_iter().filter(|t| !t.hidden))
    }

    /** make a new thread */
    pub fn new_thread<N, C>(
        &self,
        board: i64,
        author: i64,
        name: N,
        cont: C,
        file: Option<Bytes>,
    ) -> R<Thread>
    where
        N: AsRef<str>,
        C: AsRef<str>,
    {
        let time = now()?;
        let name = name.as_ref();
        let cont = cont.as_ref();
        let file = if let Some(b) = file {
            Some(self.new_file(b)?)
        } else {
            None
        };

        L!(("{} created new thread \"{name}\"", self.get_user(author)?.name) => {
            un!(self.sql.execute(
                "
                insert into threads (name, cont, hidden, file, board, time, author, locked)
                values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ",
                (name, cont, false, file.map(|f| f.id), board, time, author, false)
            ));
        });

        let r = un!(self.sql.prepare(
            "
            select * from threads
            where id = LAST_INSERT_ROWID()
            ",
        ))
        .query_map((), Thread::new)
        .map(|mut i| i.next());

        if let Some(t) = un!(r) {
            re!(t)
        } else {
            err_fmt!("Db::new_thread(): thread created, but not found")
        }
    }

    pub fn new_post<C>(
        &self,
        thread: i64,
        author: i64,
        cont: C,
        file: Option<Bytes>,
    ) -> R<Post>
    where
        C: AsRef<str>,
    {
        let time = now()?;
        let cont = cont.as_ref();
        let file = if let Some(b) = file {
            Some(self.new_file(b)?.id)
        } else {
            None
        };
        let thread = self.get_thread(thread)?;
        let board = self.get_board(thread.board)?;

        if thread.locked() {
            L!(("attempting to post in locked thread \"{}\"...erroring", thread.name) => ());
            return err_fmt!(
                "attempting to post in locked thread \"{}\"",
                thread.name
            );
        }

        L!((
            "{} made new post in thread \"{}\" with content \"{cont}\"",
            self.get_user(author)?.name,
            thread.name
        ) => {
            un!(self.sql.execute(
                "
                insert into posts (cont, hidden, file, board, thread, time, author)
                values (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ",
                (cont, false, file, board.id, thread.id, time, author)
            ));
        });

        let r = un!(self.sql.prepare(
            "
            select * from posts
            where id = LAST_INSERT_ROWID()
            "
        ))
        .query_map((), Post::new)
        .map(|mut i| i.next());

        if let Some(p) = un!(r) {
            re!(p)
        } else {
            err_fmt!("Db::new_post(): post created, but not found")
        }
    }

    pub fn get_post(&self, id: i64) -> R<Post> {
        let r = un!(self.sql.prepare(
            "
            select * from posts
            where id = ?1
            "
        ))
        .query_map((id,), Post::new)
        .map(|mut i| i.next());

        if let Some(p) = un!(r) {
            re!(p)
        } else {
            err_fmt!("Db::get_post({id}): post not found")
        }
    }

    pub fn get_theme(&self, id: i64) -> R<String> {
        let r = un!(self.sql.prepare(
            "
            select theme from users
            where id = ?1
            "
        ))
            .query_map((id,), |r| r.get(0))
            .map(|mut i| i.next());

        if let Some(t) = un!(r) {
            let o = t.unwrap_or(String::new());
            Ok(o)
        } else {
            err_fmt!("Db::get_theme({id}): theme not found")
        }
    }

    pub fn get_posts(&self, thread: i64) -> R<Vec<Post>> {
        let mut r = un!(self.sql.prepare(
            "
            select * from posts
            where thread = ?1
            "
        ));

        let map = un!(r.query_map((thread,), Post::new));
        map.map(|x| re!(x)).collect()
    }

    pub fn get_visible_posts(&self, thread: i64) -> R<Vec<Post>> {
        let mut r = un!(self.sql.prepare(
            "
            select * from posts
            where thread = ?1 and not hidden
            "
        ));

        let map = un!(r.query_map((thread,), Post::new));
        map.map(|x| re!(x)).collect()
    }

    pub fn hide_post(&self, id: i64) -> R<()> {
        L!(("hiding post {}", self.get_post(id)?.cont) => {
            un!(self.sql.execute(
                "
                update posts
                set hidden = true
                where id = ?1
                ",
                (id,)
            ));
        });

        Ok(())
    }

    pub fn update_bio<B>(&self, id: i64, bio: B) -> R<()>
    where
        B: AsRef<str>,
    {
        let bio = bio.as_ref();

        L!(("user {} updated their bio: {bio}", self.get_user(id)?.name) => {
            un!(self.sql.execute(
                "
                update users
                set bio = ?1
                where id = ?2
                ",
                (bio, id)
            ));
        });

        Ok(())
    }

    pub fn update_pass<P>(&self, id: i64, pass: P) -> R<()>
    where
        P: AsRef<str>,
    {
        let pass = pass.as_ref();
        let hash = passwd::hash(pass)?;

        L!((
            "user {} updated their password; new hash \"{hash}\"",
            self.get_user(id)?.name,
        ) => {
            un!(self.sql.execute(
                "
                update users
                set hash = ?1
                where id = ?2
                ",
                (hash, id)
            ));
        });

        Ok(())
    }

    pub fn hide_thread(&self, id: i64) -> R<()> {
        L!(("hiding thread \"{}\"", self.get_thread(id)?.name) => {
            un!(self.sql.execute(
                "
                update threads
                set hidden = true
                where id = ?1
                ",
                (id,)
            ));
        });

        Ok(())
    }

    pub fn lock_thread(&self, id: i64) -> R<()> {
        L!(("locking thread \"{}\"", self.get_thread(id)?.name) => {
            un!(self.sql.execute(
                "
                update threads
                set locked = true
                where id = ?1
                ",
                (id,)
            ));
        });

        Ok(())
    }

    pub fn unlock_thread(&self, id: i64) -> R<()> {
        L!(("unlocking thread \"{}\"", self.get_thread(id)?.name) => {
            un!(self.sql.execute(
                "
                update threads
                set locked = false
                where id = ?1
                ",
                (id,)
            ));
        });

        Ok(())
    }

    pub fn set_theme<T>(&self, id: i64, t: T) -> R<()>
    where
        T: AsRef<str>,
    {
        let t = t.as_ref();
        let u = self.get_user(id)?;

        if let Err(e) = css::get_theme(t) {
            return err_fmt!("failed to get theme \"{t}\": {e}");
        }

         L!(("setting {}'s theme to {t}", u.name) => {
            un!(self.sql.execute(
                "
                update users
                set theme = ?1
                where id = ?2
                ",
                (t, id)
            ));
        });

        Ok(())
    }
}

#[cfg(test)]
pub mod test {
    use crate::{
        db::{Db, SESSION_HASH_LEN},
        passwd,
    };
    use axum::body::Bytes;
    use std::{fs, iter::zip};

    pub fn O(p: &str) -> Db {
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

    #[test]
    fn hide_board() {
        let db = O("run/test/hide_board.db");
        let b = db.new_board("hidden", "a hidden board").unwrap();
        let b = db.hide_board(b.id).unwrap();
        assert!(b.hidden);
    }

    #[test]
    fn visible_board() {
        let db = O("run/test/visible_board.db");
        let b = db.new_board("g", "g").unwrap();
        let b = db.hide_board(b.id).unwrap();
        assert!(b.hidden);
        let b = db.visible_board(b.id).unwrap();
        assert!(!b.hidden);
    }

    #[test]
    fn visible_boards() {
        let db = O("run/test/visible_boards.db");

        /* generate some boards */
        for (n, h) in [
            ("f", false),
            ("g", false),
            ("h", true),
            ("i", false),
            ("j", true),
        ]
        .into_iter()
        {
            let b = db.new_board(n, n).unwrap();
            if h {
                db.hide_board(b.id).unwrap();
            }
        }

        let bs = db.get_visible_boards().unwrap();
        for b in bs {
            assert!(!b.hidden);
        }
    }

    #[test]
    #[should_panic]
    fn new_invalid_file() {
        let db = O("run/test/new_invalid_file.db");
        let bs = &include_bytes!("db.rs").as_slice();
        let f = db.new_file(Bytes::copy_from_slice(bs)).unwrap();

        assert_eq!(&f.bytes, bs);
    }

    #[test]
    fn new_file() {
        let db = O("run/test/new_file.db");
        let bs = &include_bytes!("crack_wires.gif").as_slice();
        let f = db.new_file(Bytes::copy_from_slice(bs)).unwrap();

        assert_eq!(bs, &f.bytes);
    }

    #[test]
    fn new_thread() {
        let db = O("run/test/new_thread.db");
        let u = db.new_user("u", "u").unwrap();
        let b = db.new_board("test", "test").unwrap();
        let t = db.new_thread(b.id, u.id, "test", "test", None).unwrap();

        assert_eq!(&t.name, "test");
        assert_eq!(&t.cont, "test");
        assert_eq!(t.hidden, false);
        assert_eq!(t.file, None);
    }

    #[test]
    fn get_threads() {
        let db = O("run/test/get_threads.db");
        let u = db.new_user("u", "u").unwrap();
        let b = db.new_board("test", "test").unwrap();
        let _ = db.new_thread(b.id, u.id, "test", "test", None).unwrap();
        let _ = db
            .new_thread(
                b.id,
                u.id,
                "test2",
                "test",
                Some(Bytes::copy_from_slice(
                    &include_bytes!("crack_wires.gif").as_slice(),
                )),
            )
            .unwrap();

        let ts = db.get_threads(b.id).unwrap();
        let [t2, t1] = &ts[..] else {
            panic!("invalid number of threads")
        };

        assert_eq!(&t1.name, "test");
        assert_eq!(&t1.cont, "test");
        assert_eq!(t1.file, None);

        assert_eq!(&t2.name, "test2");
        assert_eq!(&t2.cont, "test");
        assert!(t2.file.is_some());
    }

    #[test]
    fn new_post() {
        let db = O("run/test/new_post.db");
        let u = db.new_user("test", "test").unwrap();
        let bs = Some(Bytes::copy_from_slice(
            &include_bytes!("crack_wires.gif").as_slice(),
        ));
        let b = db.new_board("test", "test").unwrap();
        let t = db
            .new_thread(b.id, u.id, "test", "test", bs.clone())
            .unwrap();
        let p = db.new_post(t.id, u.id, "test post", bs.clone()).unwrap();
        let f = p.file.map(|i| db.get_file(i).unwrap().bytes);

        assert_eq!(p.cont, "test post");
        assert_eq!(f, bs.map(|b| b.to_vec()));
    }

    #[test]
    fn get_post() {
        let db = O("run/test/get_post.db");
        let u = db.new_user("test", "test").unwrap();
        let b = db.new_board("test", "test").unwrap();
        let t = db.new_thread(b.id, u.id, "test", "test", None).unwrap();
        let i = db.new_post(t.id, u.id, "test post", None).unwrap().id;
        let p = db.get_post(i).unwrap();
        assert_eq!(&p.cont, "test post");
        assert_eq!(p.file, None);
    }

    #[test]
    fn get_posts() {
        let db = O("run/test/get_posts.db");
        let u = db.new_user("test", "test").unwrap();
        let b = db.new_board("test", "test").unwrap();
        let t = db.new_thread(b.id, u.id, "test", "test", None).unwrap();
        let p1 = db.new_post(t.id, u.id, "test post", None).unwrap();
        let p2 = db.new_post(t.id, u.id, "test post 2", None).unwrap();
        let [p3, p4] = &db.get_posts(t.id).unwrap()[..] else {
            panic!("invalid number of posts")
        };
        assert_eq!(&p1, p3);
        assert_eq!(&p2, p4);
    }

    #[test]
    fn update_user_creds() {
        let db = O("run/test/update_user_creds.db");
        let u = db.new_user("test", "test").unwrap();
        db.update_bio(u.id, "nonempty bio").unwrap();
        let u = db.get_user(u.id).unwrap();
        assert_eq!(&u.bio, "nonempty bio");

        let u = db.new_user("test2", "test").unwrap();
        db.update_pass(u.id, "password123").unwrap();
        let u = db.get_user(u.id).unwrap();
        assert!(passwd::verify("password123", u.hash).unwrap());
    }

    #[test]
    fn hide_thread() {
        let db = O("run/test/hide_thread.db");
        let u = db.new_user("u", "u").unwrap();
        let b = db.new_board("u", "u").unwrap();
        let t = db.new_thread(b.id, u.id, "test", "test", None).unwrap();
        let _ = db.hide_thread(t.id).unwrap();
        let t = db.get_thread(t.id).unwrap();
        assert!(t.hidden);
    }

    #[test]
    fn visible_threads() {
        let db = O("run/test/visible_threads.db");
        let u = db.new_user("u", "u").unwrap();
        let b = db.new_board("f", "g").unwrap();

        for (n, h) in [
            ("a", true),
            ("b", false),
            ("c", false),
            ("d", true),
            ("e", false),
            ("f", true),
        ]
        .into_iter()
        {
            let t = db.new_thread(b.id, u.id, n, n, None).unwrap();
            if h {
                db.hide_thread(t.id).unwrap();
            }
        }

        db.get_visible_threads(b.id)
            .unwrap()
            .for_each(|t| assert!(!t.hidden));
    }

    #[test]
    fn visible_posts() {
        let db = O("run/test/visible_posts.db");
        let u = db.new_user("u", "u").unwrap();
        let b = db.new_board("f", "g").unwrap();
        let t = db.new_thread(b.id, u.id, "h", "i", None).unwrap();

        for (n, h) in [
            ("a", true),
            ("b", false),
            ("c", true),
            ("d", true),
            ("e", false),
            ("f", false),
            ("g", true),
            ("h", false),
        ]
        .into_iter()
        {
            let p = db.new_post(t.id, u.id, n, None).unwrap();
            if h {
                db.hide_post(p.id).unwrap();
            }
        }

        db.get_visible_posts(t.id)
            .unwrap()
            .into_iter()
            .for_each(|p| assert!(!p.hidden));
    }

    #[test]
    fn lock_thread() {
        let db = O("run/test/lock_thread.db");
        let u = db.new_user("a", "a").unwrap();
        let b = db.new_board("f", "g").unwrap();
        let t = db.new_thread(b.id, u.id, "h", "i", None).unwrap();
        db.lock_thread(t.id).unwrap();
        let t = db.get_thread(t.id).unwrap();
        assert!(t.locked());
    }

    #[test]
    #[should_panic]
    fn post_to_locked_thread() {
        let db = O("run/test/post_to_locked_thread.db");
        let u = db.new_user("a", "a").unwrap();
        let b = db.new_board("f", "g").unwrap();
        let t = db.new_thread(b.id, u.id, "h", "i", None).unwrap();
        db.lock_thread(t.id).unwrap();
        db.new_post(t.id, u.id, "h", None).unwrap();
    }

    #[test]
    fn unlock_thread() {
        let db = O("run/test/unlock_thread.db");
        let u = db.new_user("a", "a").unwrap();
        let b = db.new_board("f", "g").unwrap();
        let t = db.new_thread(b.id, u.id, "h", "i", None).unwrap();

        db.lock_thread(t.id).unwrap();
        let t = db.get_thread(t.id).unwrap();
        assert!(t.locked());

        db.unlock_thread(t.id).unwrap();
        let t = db.get_thread(t.id).unwrap();
        assert!(!t.locked());
    }

    #[test]
    fn set_user_theme() {
        let db = O("run/test/set_user_theme.db");
        let u = db.new_user("a", "a").unwrap();

        db.set_theme(u.id, "default").unwrap();
        let u = db.get_user(u.id).unwrap();

        assert_eq!(&u.theme.unwrap(), "default");
    }

    #[test]
    #[should_panic]
    fn set_invalid_theme() {
        let db = O("run/test/set_invalid_theme.db");
        let u = db.new_user("a", "b").unwrap();

        db.set_theme(u.id, "abcdef").unwrap();
    }

    #[test]
    fn get_theme() {
        let db = O("run/test/get_theme.db");
        let u = db.new_user("a", "a").unwrap();

        db.set_theme(u.id, "blue screen of death").unwrap();

        let t = db.get_theme(u.id).unwrap();

        assert_eq!("blue screen of death", t);
    }
}
