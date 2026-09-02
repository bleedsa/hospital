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
    let bs = &include_bytes!("../db.rs").as_slice();
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

#[test]
fn rm_admin() {
    let db = O("run/test/rm_admin.db");
    let u = db.new_user("a", "a").unwrap();

    db.new_admin(u.id).unwrap();
    let u = db.get_user(u.id).unwrap();
    assert!(u.admin);

    db.rm_admin(u.id).unwrap();
    let u = db.get_user(u.id).unwrap();
    assert!(!u.admin);
}

#[test]
fn visible_thread() {
    let db = O("run/test/visible_thread.db");
    let u = db.new_user("a", "a").unwrap();
    let b = db.new_board("a", "a").unwrap();
    let t = db.new_thread(b.id, u.id, "a", "a", None).unwrap();
    let _ = db.hide_thread(t.id).unwrap();
    let t = db.get_thread(t.id).unwrap();
    assert!(t.hidden);

    let t = db.visible_thread(t.id).unwrap();
    assert!(!t.hidden);
}

#[test]
fn visible_post() {
    let db = O("run/test/visible_post.db");
    let u = db.new_user("a", "a").unwrap();
    let b = db.new_board("a", "a").unwrap();
    let t = db.new_thread(b.id, u.id, "a", "a", None).unwrap();
    let p = db.new_post(t.id, u.id, "a", None).unwrap();
    let _ = db.hide_post(p.id).unwrap();
    let p = db.get_post(p.id).unwrap();
    assert!(p.hidden);

    let p = db.visible_post(p.id).unwrap();
    assert!(!p.hidden);
}

#[test]
fn get_replies() {
    let db = O("run/test/get_replies.db");
    let u = db.new_user("a", "a").unwrap();
    let b = db.new_board("a", "a").unwrap();
    let t = db.new_thread(b.id, u.id, "a", "a", None).unwrap();
    let p = db.new_post(t.id, u.id, "a", None).unwrap();
    let r1 = db
        .new_post(t.id, u.id, &format!(">>{}", p.id), None)
        .unwrap();
    let r2 = db
        .new_post(t.id, u.id, &format!("abcdefg >>{} 12931", p.id), None)
        .unwrap();

    let rs = db.get_post_replies(p.id).unwrap();
    assert_eq!(r1.id, rs[0]);
    assert_eq!(r2.id, rs[1]);
}

#[test]
fn get_last_post() {
    let db = O("run/test/get_last_post.db");
    let u = db.new_user("a", "a").unwrap();
    let b = db.new_board("a", "a").unwrap();
    let t = db.new_thread(b.id, u.id, "a", "a", None).unwrap();
    let _ = db.new_post(t.id, u.id, "b", None).unwrap();
    let _ = db.new_post(t.id, u.id, "c", None).unwrap();
    let _ = db.new_post(t.id, u.id, "d", None).unwrap();
    let l = db.last_post(t.id).unwrap();

    assert_eq!(&l.unwrap().cont, "d");
}

#[test]
fn mark_as_read__empty_thread() {
    let db = O("run/test/mark_as_read__empty_thread.db");
    let u = db.new_user("a", "a").unwrap();
    let b = db.new_board("a", "a").unwrap();
    let t = db.new_thread(b.id, u.id, "a", "a", None).unwrap();

    db.mark_as_read(u.id, t.id).unwrap();
    assert!(db.is_read(u.id, t.id).unwrap());
}

#[test]
fn mark_as_read__nonempty_thread() {
    let db = O("run/test/mark_as_read__nonempty_thread.db");
    let u = db.new_user("a", "a").unwrap();
    let b = db.new_board("a", "a").unwrap();
    let t = db.new_thread(b.id, u.id, "a", "a", None).unwrap();

    db.new_post(t.id, u.id, "b", None).unwrap();
    assert!(!db.is_read(u.id, t.id).unwrap());

    db.mark_as_read(u.id, t.id).unwrap();
    assert!(db.is_read(u.id, t.id).unwrap());
}

#[test]
fn new_post_is_unread() {
    let db = O("run/test/new_post_is_unread.db");
    let u = db.new_user("a", "a").unwrap();
    let b = db.new_board("a", "a").unwrap();
    let t = db.new_thread(b.id, u.id, "a", "a", None).unwrap();

    db.new_post(t.id, u.id, "b", None).unwrap();
    db.mark_as_read(u.id, t.id).unwrap();
    assert!(db.is_read(u.id, t.id).unwrap());

    db.new_post(t.id, u.id, "c", None).unwrap();
    assert!(!db.is_read(u.id, t.id).unwrap());
}

#[test]
fn add_user_css() {
    let db = O("run/test/add_user_css.db");
    let u = db.new_user("a", "a").unwrap();
    let var = "--fg:red;";
    let bod = "body{color:var(--fg);}";
    let css = db.new_css(u.id, var, bod).unwrap().unwrap();
    assert_eq!(&css.vars, var);
    assert_eq!(&css.css, bod);
}

#[test]
fn get_css__no_css() {
    let db = O("run/test/get_css__no_css.db");
    let u = db.new_user("a", "a").unwrap();
    let css = db.get_css(u.id).unwrap();
    assert_eq!(css, None);
}

#[test]
fn get_css__several_css() {
    let db = O("run/test/get_css__several_css.db");
    let u = db.new_user("a", "a").unwrap();
    let _ = db.new_css(u.id, "", "test").unwrap().unwrap();

    let css = db.new_css(u.id, "", "test2").unwrap().unwrap();
    assert_eq!(&css.css, "test2");
}

#[test]
fn set_board_name() {
    let db = O("run/test/set_board_name.db");
    let b = db.new_board("a", "a").unwrap();
    db.update_board_name(b.id, "b").unwrap();
    let b = db.get_board(b.id).unwrap();

    assert_eq!(&b.name, "b");
}
