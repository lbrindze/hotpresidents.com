use std::sync::Mutex;

use actix_files as fs;
use actix_web::http::{header, StatusCode};
use actix_web::{get, guard, middleware, web, App, HttpResponse, HttpServer, Responder, Result};
use askama::Template;
use rand::seq::SliceRandom;

mod at_client;
mod config;
mod models;
mod templates;

use crate::at_client::reload_airtables;
use crate::models::load_state;
use crate::models::*;
use crate::templates::*;

/// save count data
#[get("/save_data")]
async fn save_data(presidents: web::Data<Mutex<Presidents>>) -> HttpResponse {
    let presidents = presidents.lock().unwrap();

    save_state(&presidents).await;

    HttpResponse::Found()
        .header(header::LOCATION, "/")
        .finish()
        .into_body()
}

/// Trigger data reload
#[get("/reload_data")]
async fn reload_data(presidents: web::Data<Mutex<Presidents>>) -> HttpResponse {
    let mut presidents = presidents.lock().unwrap();

    reload_airtables(&mut presidents).await;

    HttpResponse::Found()
        .header(header::LOCATION, "/")
        .finish()
        .into_body()
}

/// 404 handler
async fn p404() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
}

#[get("/vote/{id}")]
async fn vote(
    presidents: web::Data<Mutex<Presidents>>,
    web::Path(id): web::Path<String>,
) -> impl Responder {
    let presidents_idx = presidents.lock().unwrap();
    let s = if let Some(president) = presidents_idx.get(&id) {
        VoteTemplate {
            name: &president.name,
            short_name: &president.short_name(),
            office: &president.office,
            image_url: match &president.images.get(0) {
                Some(image) => image.thumbnails.large.url.as_str(),
                None => "",
            },
            years_in_office: &president
                .years_in_office
                .as_ref()
                .unwrap_or(&"".to_string()),
            quote: match president.quote.as_ref() {
                Some(quote) => quote.as_str(),
                None => "",
            },
        }
        .render()
        .unwrap()
    } else {
        // TODO: handle error correctly....
        return HttpResponse::Found()
            .header(header::LOCATION, "/error")
            .finish()
            .into_body();
    };

    HttpResponse::Ok().content_type("text/html").body(s)
}

#[get("/stats/{id}")]
async fn stats(
    presidents: web::Data<Mutex<Presidents>>,
    web::Path(id): web::Path<String>,
) -> impl Responder {
    let presidents_idx = presidents.lock().unwrap();
    let s = if let Some(president) = presidents_idx.get(&id) {
        StatsTemplate {
            name: &president.name,
            office: &president.office,
            years_in_office: &president
                .years_in_office
                .as_ref()
                .unwrap_or(&"".to_string()),
            hot: &president.hot,
            not: &president.not,
            image_url: match &president.images.get(0) {
                Some(image) => image.thumbnails.large.url.as_str(),
                None => "",
            },
            quote: president.quote.as_ref().unwrap_or(&"".to_string()),
        }
        .render()
        .unwrap()
    } else {
        // TODO: handle error correctly....
        return HttpResponse::Found()
            .header(header::LOCATION, "/error")
            .finish()
            .into_body();
    };

    HttpResponse::Ok().content_type("text/html").body(s)
}

#[get("/vote/{id}/{adj}")]
async fn cast_vote(
    presidents: web::Data<Mutex<Presidents>>,
    web::Path((id, adj)): web::Path<(String, String)>,
) -> impl Responder {
    let mut presidents_idx = presidents.lock().unwrap();
    let mut president = presidents_idx.get(&id).unwrap().clone();

    match adj.as_str() {
        "hot" => president.hot_vote(),
        "not" => president.not_vote(),
        _ => {}
    };

    presidents_idx.insert(president.short_name(), president);

    HttpResponse::Found()
        .header(header::LOCATION, format!("/stats/{}", &id))
        .finish()
        .into_body()
}

#[get("/")]
async fn next_president(presidents: web::Data<Mutex<Presidents>>) -> HttpResponse {
    let presidents = presidents.lock().unwrap();

    let keys = presidents.keys().collect::<Vec<&String>>();
    let next = keys.choose(&mut rand::thread_rng()).unwrap();

    HttpResponse::Found()
        .header(header::LOCATION, format!("/vote/{}", next))
        .finish()
        .into_body()
}

#[get("/index.html")]
async fn index(presidents: web::Data<Mutex<Presidents>>) -> HttpResponse {
    let presidents = presidents.lock().unwrap();
    let s = IndexTemplate {
        presidents: to_index_items(&presidents),
    }
    .render()
    .unwrap();

    HttpResponse::Ok().content_type("text/html").body(s)
}

/// favicon handler
#[get("/favicon")]
async fn favicon() -> Result<fs::NamedFile> {
    Ok(fs::NamedFile::open("static/favicon.ico")?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut presidents = Presidents::new();
    reload_airtables(&mut presidents).await;
    load_state(&mut presidents).await;

    let data = web::Data::new(Mutex::new(presidents));

    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    log::info!("Logger Initialized, Starting Server..");

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(middleware::Logger::default())
            .service(vote)
            .service(cast_vote)
            .service(reload_data)
            .service(save_data)
            .service(stats)
            .service(index)
            .service(next_president)
            .service(favicon)
            .service(fs::Files::new("/static", "static").show_files_listing())
            .default_service(
                // 404 for GET request
                web::resource("")
                    .route(web::get().to(p404))
                    // all requests that are not `GET`
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
