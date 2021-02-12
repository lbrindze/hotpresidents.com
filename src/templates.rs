use crate::models::PresidentIndexItem;
use askama::Template;

#[derive(Template)]
#[template(path = "vote.html")]
pub struct VoteTemplate<'a> {
    pub name: &'a str,
    pub short_name: &'a str,
    pub image_url: &'a str,
    pub office: &'a str,
    pub years_in_office: &'a str,
    pub quote: &'a str,
}

#[derive(Template)]
#[template(path = "stats.html")]
pub struct StatsTemplate<'a> {
    pub name: &'a str,
    pub image_url: &'a str,
    pub quote: &'a str,
    pub office: &'a str,
    pub years_in_office: &'a str,
    pub hot: &'a usize,
    pub not: &'a usize,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub presidents: Vec<PresidentIndexItem>,
}
