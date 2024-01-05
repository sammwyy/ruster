use std::fmt;

use clap::ValueEnum;
use colored::Colorize;
use futures::{stream, StreamExt};
use reqwest::{header::HeaderMap, Client};
use tokio;

use crate::utils::random_element;

#[derive(ValueEnum, Clone, Default, Debug)]
pub enum Mode {
    // Search for files or directories in the target
    #[default]
    Dir,
    // Fuzz the target with the wordlist, replacing {value} with the word
    Fuzz,
    // Search for Virtual-Hosts in the target
    VHost,
    // Search for Subdomains in the target
    DNS,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Dir => write!(f, "Dir (Directory)"),
            Mode::Fuzz => write!(f, "Fuzz (Fuzzing)"),
            Mode::VHost => write!(f, "VHost (Virtual Host)"),
            Mode::DNS => write!(f, "DNS (Subdomains)"),
        }
    }
}

pub async fn start_buster(
    mode: Mode,
    target: String,
    headers: HeaderMap,
    wordlist: Vec<String>,
    uas: Vec<String>,
    threads: usize,
) {
    let client = Client::builder().default_headers(headers).build().unwrap();

    let bodies = stream::iter(wordlist)
        .map(|value| {
            let client = client.clone();
            let full_url = target.replace("{value}", &value);
            let ua = random_element(uas.clone());

            tokio::spawn(async move {
                let mut req = client.get(&full_url);

                if ua.is_some() {
                    req = req.header("User-Agent", ua.unwrap());
                }

                let resp = req.send().await?;
                let status_code = resp.status().as_u16();

                if status_code != 404 && status_code != 400 {
                    let status_display = format!("{}", status_code).green();
                    println!(
                        "{} {} {} ({})",
                        ">".magenta(),
                        "Found:".cyan(),
                        full_url,
                        status_display
                    );
                }

                resp.bytes().await
            })
        })
        .buffer_unordered(threads);

    bodies.for_each(|_| async {}).await;
}
