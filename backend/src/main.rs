#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use rocket::http::{Cookie, Cookies};
use rocket::request::Form;
use rocket::response::{Debug, NamedFile, Redirect};
use rocket::State;
use rocket_contrib::templates::Template;

use std::io;
use std::path::{Path, PathBuf};
use std::thread;

mod http_to_https;
mod session_manager;

use session_manager::{AdminId, Id, ParticipantId, Sessions};

static WWW_DIR: &'static str = "www";

#[get("/<file..>")]
fn static_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new(WWW_DIR).join(file)).ok()
}
#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open(Path::new(WWW_DIR).join("index.html"))
}

fn already_logged_in_template(id: Id, sessions: State<Sessions>) -> Template {
    match id {
        Id::Participant(pid) => {
            Template::render("already_logged_in", sessions.get_participant(pid).clone())
        }
        Id::Admin(aid) => {
            Template::render("already_logged_in_admin", sessions.get_admin(aid).clone())
        }
    }
}
#[post("/login")]
fn already_logged_in(id: Id, sessions: State<Sessions>) -> Template {
    already_logged_in_template(id, sessions)
}
#[post("/admin_login")]
fn already_logged_in_admin(id: Id, sessions: State<Sessions>) -> Template {
    already_logged_in_template(id, sessions)
}

#[derive(rocket::request::FromForm)]
struct LoginData {
    name: String,
    school: String,
    grade: u8,
}

#[post("/login", data = "<login_data>", rank = 2)]
fn login(mut cookies: Cookies, sessions: State<Sessions>, login_data: Form<LoginData>) -> Redirect {
    let LoginData {
        name,
        school,
        grade,
    } = login_data.into_inner();
    let id = sessions.new_participant(name, school, grade);
    cookies.add(Cookie::new("id", Id::Participant(id).to_string()));
    Redirect::to("/welcome")
}

#[derive(rocket::request::FromForm)]
struct AdminLoginData {
    school: String,
}

#[post("/admin_login", data = "<login_data>", rank = 2)]
fn admin_login(
    mut cookies: Cookies,
    sessions: State<Sessions>,
    login_data: Form<AdminLoginData>,
) -> Redirect {
    let AdminLoginData { school } = login_data.into_inner();
    let id = sessions.new_admin(school);
    cookies.add(Cookie::new("id", Id::Admin(id).to_string()));
    Redirect::to("/welcome")
}

#[get("/logout")]
fn logout(_id: Id, mut cookies: Cookies) -> Redirect {
    cookies.remove(Cookie::named("id"));
    Redirect::to("/")
}

#[get("/welcome")]
fn welcome(id: Id, sessions: State<Sessions>) -> Template {
    match id {
        Id::Participant(pid) => {
            let participant = sessions.get_participant(pid).clone();
            Template::render("welcome", participant)
        }
        Id::Admin(aid) => {
            let admin = sessions.get_admin(aid).clone();
            Template::render("welcome_admin", admin)
        }
    }
}

fn clear_cookies(cookies: &mut Cookies) {
    let to_remove = cookies
        .iter()
        .map(|cook| Cookie::named(cook.name().to_string()))
        .collect::<Vec<Cookie>>();
    for cook in to_remove {
        cookies.remove(cook);
    }
}

#[get("/clear_cookies?<uri>")]
fn clear_cookies_page(mut cookies: Cookies, uri: String) -> Redirect {
    clear_cookies(&mut cookies);
    Redirect::to(uri)
}

#[get("/clear_cookies")]
fn clear_cookies_page_noredir(mut cookies: Cookies) -> Redirect {
    clear_cookies(&mut cookies);
    Redirect::to("/")
}

fn main() {
    thread::spawn(|| {
        http_to_https::Config::new()
            .set_http_port(8080)
            .set_https_port(4443)
            .serve();
    });

    rocket::ignite()
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                static_files,
                index,
                login,
                admin_login,
                already_logged_in,
                already_logged_in_admin,
                welcome,
                logout,
                clear_cookies_page_noredir,
                clear_cookies_page
            ],
        )
        .manage(Sessions::new())
        .launch();
}
