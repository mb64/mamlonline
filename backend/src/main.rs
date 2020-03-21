#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use rocket::http::{Cookie, Cookies};
use rocket::request::Form;
use rocket::response::{Debug, NamedFile, Redirect};
use rocket_contrib::templates::Template;

use std::io;
use std::path::{Path, PathBuf};
use std::thread;

mod http_to_https;

static WWW_DIR: &'static str = "www";

#[get("/<file..>")]
fn static_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new(WWW_DIR).join(file)).ok()
}

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open(Path::new(WWW_DIR).join("index.html"))
}

#[derive(rocket::request::FromForm, Serialize, Deserialize)]
struct LoginData {
    school: String,
    name: String,
    grade: u8,
}

#[post("/login", data = "<login_data>")]
fn login(
    mut cookies: Cookies,
    login_data: Form<LoginData>,
) -> Result<Redirect, Debug<serde_json::Error>> {
    cookies.add(Cookie::new(
        "login",
        serde_json::to_string(&login_data.into_inner())?,
    ));
    Ok(Redirect::to("/welcome"))
}

#[get("/welcome")]
fn welcome(cookies: Cookies) -> Option<Result<Template, Debug<serde_json::Error>>> {
    Some(
        serde_json::from_str(&cookies.get("login")?.value())
            .map_err(|e| e.into())
            .map(|login_data: LoginData| Template::render("welcome", login_data)),
    )
}

#[get("/clear_cookies?<uri>")]
fn clear_cookies(mut cookies: Cookies, uri: String) -> Redirect {
    let to_remove = cookies
        .iter()
        .map(|cook| Cookie::named(cook.name().to_string()))
        .collect::<Vec<Cookie>>();
    for cook in to_remove {
        cookies.remove(cook);
    }
    Redirect::to(uri)
}

#[get("/clear_cookies")]
fn clear_cookies_noredir() -> Redirect {
    Redirect::to("/clear_cookies?uri=/")
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
                welcome,
                clear_cookies_noredir,
                clear_cookies
            ],
        )
        .launch();
}
