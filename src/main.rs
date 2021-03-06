extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;
extern crate chrono;
extern crate dotenv;
extern crate uuid;
extern crate serde;
extern crate serde_json;
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate actix;
extern crate actix_web;
extern crate futures;

use r2d2_sqlite::SqliteConnectionManager;
use dotenv::dotenv;
use std::env;
use std::path::Path;
use actix::prelude::*;
use actix_web::http::{Method};
use actix_web::{
    http, middleware, server, App, AsyncResponder, FutureResponse, HttpResponse, 
    State, HttpRequest, fs
};

use futures::Future;

mod db;
use db::{DbExecutor, Queries, Pool, SampleQuery}; 

struct AppState {
    db: Addr<Syn, DbExecutor>
}

/// Version 1: Calls 4 queries in sequential order, as an asynchronous handler
fn get_samples(state: State<AppState>) -> FutureResponse<HttpResponse> {
    println!("samples request");
    state.db.send(Queries::GetAllSamples).from_err()
    .and_then(|res| match res {
        Ok(samples) => Ok(HttpResponse::Ok().json(samples)),
        Err(_) => Ok(HttpResponse::InternalServerError().into())
    })
    .responder()
}

/// Version 1: Calls 4 queries in sequential order, as an asynchronous handler
fn query_samples(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    println!("samples request");
    println!("Query string: ${:?}", req.query_string());
    // let limit = req.match_info().query("limit").unwrap();
    // println!("${:?}", limit);

    req.state().db.send(SampleQuery{
        query_string: req.query_string().to_string()
    }).from_err()
    .and_then(|res| match res {
        Ok(samples) => Ok(HttpResponse::Ok().json(samples)),
        Err(_) => Ok(HttpResponse::InternalServerError().into())
    })
    .responder()
}

fn main() {
    dotenv().ok();
    // The statements here will be executed when the compiled binary is called
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_path = Path::new(&database_url);
    if !db_path.exists() {
        panic!("{:?} does not exist!", db_path);
    }
    let sys = actix::System::new("sample-api");
    println!("Opening DB at: {:?}", db_path);
    //let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let manager = SqliteConnectionManager::file(&database_url); // , OpenFlags::SQLITE_OPEN_READ_ONLY);
    let pool = Pool::new(manager).unwrap();


    let static_page_folder = env::var("STATIC_PAGE_FOLDER").expect("STATIC_PAGE_FOLDER must be set");
    let addr = SyncArbiter::start(3, move || DbExecutor(pool.clone()));

    server::new(move || {
        App::with_state(AppState{db: addr.clone()})
            // enable logger
            .middleware(middleware::Logger::default())
            .resource("/test", |r| r.f(|req| {
                match *req.method() {
                    Method::GET => HttpResponse::Ok(),
                    Method::POST => HttpResponse::MethodNotAllowed(),
                    _ => HttpResponse::NotFound(),
                }
            }))
            .resource("/query_samples", |r| r.method(http::Method::GET).f(query_samples))
            .resource("/samples", |r| r.method(http::Method::GET).with(get_samples))
            .handler(
                "/",
                fs::StaticFiles::new(&static_page_folder).index_file("index.html"))
    }).bind("0.0.0.0:4200")
        .unwrap()
        .start();

    println!("Started http server: 0.0.0.0:4200");
    let _ = sys.run();
}
