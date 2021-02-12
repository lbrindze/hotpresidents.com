use actix_web::web;
use serde::{self, Deserialize};

use std::collections::HashMap;
use std::default::Default;
use std::fs::File;
use std::io::BufRead;
use std::io::{self, Write};
use std::path::Path;

use crate::config;

///
/// Internal data models
///

#[derive(Clone, Debug, Deserialize, Default)]
pub struct President {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Office")]
    pub office: String,
    #[serde(rename = "Party")]
    pub party: String,
    #[serde(rename = "Quote")]
    pub quote: Option<String>,
    #[serde(rename = "Years in Office")]
    pub years_in_office: Option<String>,
    #[serde(rename = "Year took Office")]
    pub term_year: u16,
    #[serde(rename = "Number in Office")]
    pub term_length: u16,
    #[serde(rename = "Attachments")]
    pub images: Vec<PresidentImage>,
    #[serde(default)]
    pub hot: usize,
    #[serde(default)]
    pub not: usize,
}

impl President {
    pub fn hot_vote(&mut self) {
        self.hot += 1
    }

    pub fn not_vote(&mut self) {
        self.not += 1
    }

    pub fn score(&self) -> isize {
        (self.hot as isize) - (self.not as isize)
    }

    pub fn short_name(&self) -> String {
        self.name.to_lowercase().replace(".", "").replace(" ", "_")
    }

    pub fn template_item(&self) -> PresidentIndexItem {
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

pub struct PresidentIndexItem {
    pub name: String,
    pub short_name: String,
    pub score: isize,
    pub image_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PresidentImage {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub size: usize,
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(default)]
    pub thumbnails: Thumbnails,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct Thumbnails {
    pub small: ImageThumbnail,
    pub large: ImageThumbnail,
    pub full: ImageThumbnail,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct ImageThumbnail {
    pub url: String,
    pub width: u16,
    pub height: u16,
}

pub type Presidents = HashMap<String, President>;

pub fn to_index_items(p: &Presidents) -> Vec<PresidentIndexItem> {
    p.values().map(|p| p.template_item()).collect()
}

pub async fn save_state(presidents: &Presidents) {
    // File::create is blocking operation, use threadpool
    let cfg = config::from_envvar();
    let mut f = web::block(|| std::fs::File::create(cfg.save_file))
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

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub async fn load_state(presidents: &mut Presidents) {
    let cfg = config::from_envvar();
    if let Ok(lines) = read_lines(cfg.save_file) {
        for line in lines {
            let line = line.unwrap();

            let mut splitter = line.split(",");

            let key = splitter.next().unwrap();
            let hot = splitter.next().unwrap();
            let not = splitter.next().unwrap();

            let president = presidents
                .entry(key.to_string())
                .or_insert(Default::default());
            president.hot = hot.parse().unwrap();
            president.not = not.parse().unwrap();
        }
    }
}
