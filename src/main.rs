use std::collections::HashMap;
use std::io::Write;
use std::sync::Mutex;

use actix_files as fs;
use actix_web::client::Client;
use actix_web::http::{header, StatusCode};
use actix_web::{get, guard, middleware, web, App, HttpResponse, HttpServer, Responder, Result};

use rand::seq::SliceRandom;

use askama::Template;
use serde::{self, Deserialize};

const DATA_LOAD_URI: &str =
    "https://api.airtable.com/v0/appWPQd75Wh8IVPa0/Table%201?view=Grid%20view";
const API_KEY: &str = "keybq7TXDnBmxHzBV";
const SAVE_FILE: &str = "president_votes_state.data";

///
/// Internal data models
///

#[derive(Clone, Debug, Deserialize)]
struct PresidentImage {
    #[serde(default)]
    url: String,
    #[serde(default)]
    filename: String,
    #[serde(default)]
    size: usize,
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    thumbnails: Thumbnails,
}

#[derive(Clone, Debug, Deserialize, Default)]
struct Thumbnails {
    small: ImageThumbnail,
    large: ImageThumbnail,
    full: ImageThumbnail,
}

#[derive(Clone, Debug, Deserialize, Default)]
struct ImageThumbnail {
    url: String,
    width: u16,
    height: u16,
}

#[derive(Clone, Debug, Deserialize)]
struct President {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Office")]
    office: String,
    #[serde(rename = "Party")]
    party: String,
    #[serde(rename = "Year took Office")]
    term_year: u16,
    #[serde(rename = "Number in Office")]
    term_length: u16,
    #[serde(rename = "Attachments")]
    images: Vec<PresidentImage>,
    #[serde(default)]
    hot: usize,
    #[serde(default)]
    not: usize,
}

impl President {
    fn hot_vote(&mut self) {
        self.hot += 1
    }

    fn not_vote(&mut self) {
        self.not += 1
    }

    fn score(&self) -> isize {
        (self.hot as isize) - (self.not as isize)
    }

    fn short_name(&self) -> String {
        self.name.to_lowercase().replace(".", "").replace(" ", "_")
    }

    fn template_item(&self) -> PresidentIndexItem {
        PresidentIndexItem {
            name: self.name.to_string(),
            short_name: self.short_name(),
            score: self.score(),
            image_url: match self.images.get(0) {
                Some(image) => image.thumbnails.large.url.to_string(),
                None => "".to_string(),
            },
        }
    }
}

type Presidents = HashMap<String, President>;

fn to_index_items(p: &Presidents) -> Vec<PresidentIndexItem> {
    p.values().map(|p| p.template_item()).collect()
}

async fn save_state(presidents: &Presidents) {
    // File::create is blocking operation, use threadpool
    let mut f = web::block(|| std::fs::File::create(SAVE_FILE))
        .await
        .unwrap();

    for president in presidents.values() {
        let data = format!(
            "{},{},{}\n",
            president.short_name(),
            president.hot,
            president.not
        );
        f = web::block(move || f.write_all(&data.as_bytes()).map(|_| f))
            .await
            .unwrap();
    }
}

/*
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

async fn load_state(presidents: &Presidents) {
    if let Ok(lines) = read_lines("./hosts") {
    }
}
*/

///
/// Client for working with Airtables
///

#[derive(Debug, Deserialize)]
struct ResponseItem {
    id: String,
    fields: President,
}

#[derive(Debug, Deserialize)]
struct Payload {
    records: Vec<ResponseItem>,
}

async fn get_airtables_data(client: &Client) -> Vec<President> {
    // Initialize presidents return data struct
    let mut presidents = vec![];

    // Create request builder and send request
    log::info!("Calling Airtables API");
    let response = client
        .get(DATA_LOAD_URI)
        .header("User-Agent", "actix-web/3.0")
        .header("Authorization", format!("Bearer {}", API_KEY))
        .send()
        .await;

    match response {
        Ok(mut resp) => {
            let json_content: Payload = resp.json().limit(250_000).await.unwrap();

            for item in json_content.records {
                let president = item.fields;
                presidents.push(president);
            }
        }
        Err(err) => {
            log::warn!("Server Error, could not get data from data store");
            log::error!("Fatal! {}", err);
        }
    };

    presidents
}

async fn reload_airtables(presidents: &mut Presidents) {
    let client = Client::default();
    let data: Vec<President> = get_airtables_data(&client).await;

    for president in data {
        presidents.insert(president.short_name(), president);
    }
}

///
/// Templates
///
#[derive(Template)]
#[template(path = "vote.html")]
struct VoteTemplate<'a> {
    name: &'a str,
    short_name: &'a str,
    image_url: &'a str,
}

#[derive(Template)]
#[template(path = "stats.html")]
struct StatsTemplate<'a> {
    name: &'a str,
    image_url: &'a str,
    hot: &'a usize,
    not: &'a usize,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    presidents: Vec<PresidentIndexItem>,
}

struct PresidentIndexItem {
    name: String,
    short_name: String,
    score: isize,
    image_url: String,
}

///
/// Routes
///

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
            image_url: match &president.images.get(0) {
                Some(image) => image.thumbnails.large.url.as_str(),
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
            image_url: match &president.images.get(0) {
                Some(image) => image.thumbnails.large.url.as_str(),
                None => "",
            },
            hot: &president.hot,
            not: &president.not,
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut presidents = Presidents::new();
    reload_airtables(&mut presidents).await;

    let data = web::Data::new(Mutex::new(presidents));

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

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
