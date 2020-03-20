#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::response::NamedFile;

use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::thread;

mod http_to_https;

static WWW_DIR: &'static str = "../www";

#[get("/<file..>")]
fn static_files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new(WWW_DIR).join(file)).ok()
}

#[get("/")]
fn index() -> io::Result<NamedFile> {
    NamedFile::open(Path::new(WWW_DIR).join("index.html"))
}

fn main() {
    thread::spawn(|| {
        http_to_https::Config::new()
            .set_http_port(8080)
            .set_https_port(4443)
            .serve();
    });

    rocket::ignite()
        .mount("/", routes![static_files, index])
        .launch();
}
