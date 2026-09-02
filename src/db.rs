use axum::body::Bytes;
use axum_cookie::prelude::*;
use image::ImageReader;
use rusqlite::{Connection, Row, params};

use crate::{css, passwd, pre::*, rand::rand_str};
use std::{fmt, fs, io::Cursor, path::Path};

#[cfg(test)]
mod test;

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

impl User {
    pub fn fields() -> &'static str {
        "id|name|hash|bio|admin|theme"
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{:?}",
            self.id, self.name, self.hash, self.bio, self.admin, self.theme
        )
    }
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

    pub fn fields() -> &'static str {
        "id|name|desc|hidden"
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}|{}|{}|{}", self.id, self.name, self.desc, self.hidden)
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

    pub fn fields() -> &'static str {
        "id|name|content|hidden|file|board|time|author|locked"
    }
}

impl fmt::Display for Thread {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}|{:?}|{:?}|{}|{:?}|{}|{}|{}|{}",
            self.id,
            self.name,
            self.cont,
            self.hidden,
            self.file,
            self.board,
            self.time,
            self.author,
            self.locked()
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Read {
    pub id: i64,
    pub marked: i64,
    pub thread: i64,
    pub time: i64,
    pub is_post: bool,
    pub user: i64,
}

impl Read {
    #[inline(always)]
    pub fn new<'a>(r: &Row<'a>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: r.get(0)?,
            marked: r.get(1)?,
            thread: r.get(2)?,
            time: r.get(3)?,
            is_post: r.get(4)?,
            user: r.get(5)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Css {
    pub id: i64,
    pub user: i64,
    pub css: String,
    pub vars: String,
}

impl Css {
    pub fn new<'a>(r: &Row<'a>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: r.get(0)?,
            user: r.get(1)?,
            css: r.get(2)?,
            vars: r.get(3)?,
        })
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

    pub fn fields() -> &'static str {
        "id|cont|hidden|file|board|thread|time|author"
    }
}

impl fmt::Display for Post {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}|{:?}|{}|{:?}|{}|{}|{}|{}",
            self.id,
            self.cont,
            self.hidden,
            self.file,
            self.board,
            self.thread,
            self.time,
            self.author
        )
    }
}

pub struct Db {
    pub sql: Connection,
    pub path: String,
}

impl Db {
    pub fn create<P>(_p: P) -> R<Self>
    where
        P: AsRef<str> + ToString,
    {
        let p = Path::new(_p.as_ref());

        /* if the run dir does not exist, we create it for you,
         * little special snowflake. */
        if let Some(d) = p.parent()
            && !un!(fs::exists(d))
        {
            un!(fs::create_dir_all(d));
        }

        /* open */
        let sql = un!(Connection::open(p));
        Ok(Self {
            sql,
            path: _p.to_string(),
        })
    }

    #[inline(always)]
    pub fn new() -> R<Self> {
        Db::create(&(&*CFG).server.db)
    }

    pub fn new_opt<P>(o: Option<P>) -> R<Self>
    where
        P: AsRef<str> + ToString
    {
        if let Some(p) = o {
            Self::create(p)?.init()
        } else {
            Self::new()
        }
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
            r#"
            create table if not exists read (
                id integer primary key autoincrement,
                marked integer not null,
                thread integer not null,
                time integer not null,
                is_post boolean not null,
                user integer not null
            );
            "#,
            r#"
            create table if not exists css (
                id integer primary key autoincrement,
                user integer not null,
                css text not null,
                vars text not null
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

    /** get all admin users */
    pub fn get_all_admins(&self) -> R<Vec<User>> {
        let mut r = un!(self.sql.prepare(
            "
            select * from users
            where admin
            "
        ));

        un!(r.query_map((), row2user)).map(|x| re!(x)).collect()
    }

    pub fn get_all_users(&self) -> R<Vec<User>> {
        let mut r = un!(self.sql.prepare("select * from users"));
        un!(r.query_map((), row2user)).map(|x| re!(x)).collect()
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

    /** make a new read indicator entry */
    pub fn mark_as_read(&self, u: i64, t: i64) -> R<()> {
        let l = self.last_post(t)?;
        if let Some(p) = l {
            un!(self.sql.execute(
                "
                insert into read (marked, time, thread, is_post, user)
                values (?1, ?2, ?3, true, ?4)
                ",
                (p.id, now()?, t, u),
            ));
        } else {
            un!(self.sql.execute(
                "
                insert into read (marked, time, thread, is_post, user)
                values (?1, ?2, ?3, false, ?4)
                ",
                (t, now()?, t, u)
            ));
        }

        Ok(())
    }

    /** is this thread read? */
    pub fn is_read(&self, u: i64, t: i64) -> R<bool> {
        let last = self.last_post(t)?;

        Ok(if let Some(last) = last {
            if let Some(read) = self.last_read(t)? {
                read.user == u && read.marked == last.id
            } else {
                false
            }
        } else {
            if let Some(read) = self.last_read(t)? {
                read.user == u && read.marked == t
            } else {
                false
            }
        })
    }

    /** get the last read marker */
    pub fn last_read(&self, t: i64) -> R<Option<Read>> {
        let r = un!(self.sql.prepare(
            "
            select * from read
            where thread = ?1
            order by time
            "
        ))
        .query_map((t,), Read::new)
        .map(|i| i.last());

        Ok(if let Some(x) = un!(r) {
            let u = un!(x);
            Some(u)
        } else {
            None
        })
    }

    /** get the last post made in a thread */
    pub fn last_post(&self, t: i64) -> R<Option<Post>> {
        let r = un!(self.sql.prepare(
            "
            select * from posts
            where thread = ?1
            order by time
            "
        ))
        .query_map((t,), Post::new)
        .map(|i| i.last());

        Ok(if let Some(x) = un!(r) {
            let u = un!(x);
            Some(u)
        } else {
            None
        })
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
        let b = self.get_board(id)?;

        L!(("making board \"{}\" visible", b.name) => {
            un!(self.sql.execute(
                "
                update boards
                set hidden = false
                where id = ?1
                ",
                (id,)
            ))
        });

        self.get_board(id)
    }

    /** make a thread visible */
    pub fn visible_thread(&self, id: i64) -> R<Thread> {
        let t = self.get_thread(id)?;

        L!(("making thread \"{}\"({}) visible", t.name, t.id) => {
            un!(self.sql.execute(
                "
                update threads
                set hidden = false
                where id = ?1
                ",
                (id,)
            ));
        });

        self.get_thread(id)
    }

    /** make a post visible */
    pub fn visible_post(&self, id: i64) -> R<Post> {
        let p = self.get_post(id)?;

        L!(("making post {id} visible (\"{}\")", p.cont) => {
            un!(self.sql.execute(
                "
                update posts
                set hidden = false
                where id = ?1
                ",
                (id,)
            ));
        });

        self.get_post(id)
    }

    /** add a new file to the db */
    pub fn new_file(&self, file: Bytes) -> R<File> {
        let vec = file.to_vec();

        /* verify that the file is, in fact, an image */
        let im = un!(ImageReader::new(Cursor::new(&vec)).with_guessed_format());
        let _ = un!(im.decode());

        L!(("creating new file") => {
            un!(self.sql.execute(
                "
                insert into files (bytes)
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

    pub fn new_css<V, C>(&self, uid: i64, vars: V, css: C) -> R<Option<Css>>
    where
        V: AsRef<str>,
        C: AsRef<str>,
    {
        let vars = vars.as_ref();
        let css = css.as_ref();

        L!(("creating new css for user {}: {css}", self.get_user(uid)?.name) => {
            un!(self.sql.execute(
                "
                insert into css (user, css, vars)
                values (?1, ?2, ?3)
                ",
                (uid, css, vars),
            ))
        });

        self.get_css(uid)
    }

    pub fn get_css(&self, uid: i64) -> R<Option<Css>> {
        let r = un!(self.sql.prepare(
            "
            select * from css
            where user = ?1
            "
        ))
        .query_map((uid,), Css::new)
        .map(|i| i.last());

        if let Some(c) = un!(r) {
            Ok(Some(un!(c)))
        } else {
            Ok(None)
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
        for t in vec.into_iter() {
            let mut times = self
                .get_posts(t.id)?
                .into_iter()
                .map(|p| p.time)
                .collect::<Vec<_>>();
            times.sort();
            /* safety */
            if let Some(e) = times.last() {
                zipped.push((*e, t));
            } else {
                zipped.push((t.time, t));
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

    pub fn get_post_replies(&self, id: i64) -> R<Vec<i64>> {
        let mut r = un!(self.sql.prepare(
            "
            select id from posts
            where cont like concat('%',?1,'%')
            "
        ));

        un!(r.query_map((id.to_string(),), |r| r.get(0)))
            .map(|x| re!(x))
            .collect()
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

    pub fn get_all_hidden_threads(&self) -> R<Vec<Thread>> {
        let mut r = un!(self.sql.prepare(
            "
            select * from threads
            where hidden
            "
        ));

        un!(r.query_map((), Thread::new)).map(|x| re!(x)).collect()
    }

    pub fn get_all_hidden_posts(&self) -> R<Vec<Post>> {
        let mut r = un!(self.sql.prepare(
            "
            select * from posts
            where hidden
            "
        ));

        un!(r.query_map((), Post::new)).map(|x| re!(x)).collect()
    }

    pub fn get_all_hidden_boards(&self) -> R<Vec<Board>> {
        let mut r = un!(self.sql.prepare(
            "
            select * from boards
            where hidden
            "
        ));

        un!(r.query_map((), Board::new)).map(|x| re!(x)).collect()
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

    pub fn rm_admin(&self, id: i64) -> R<()> {
        un!(self.sql.execute(
            "
            update users
            set admin = false
            where id = ?1
            ",
            (id,)
        ));

        Ok(())
    }
}
