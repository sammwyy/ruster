use std::fmt;

use clap::ValueEnum;
use colored::Colorize;
use futures::{stream, StreamExt};
use reqwest::{header::HeaderMap, Client};
use tokio;

use crate::utils::random_element;

#[derive(ValueEnum, Clone, Default, Debug, PartialEq)]
pub enum Mode {
    // Search for files or directories in the target
    #[default]
    Dir,
    // Fuzz the target with the wordlist, replacing {fuzz} with the word
    Fuzz,
    // Search for Virtual-Hosts in the target
    Vhost,
    // Search for Subdomains in the target
    DNS,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Mode::Dir => write!(f, "Dir (Directory)"),
            Mode::Fuzz => write!(f, "Fuzz (Fuzzing)"),
            Mode::Vhost => write!(f, "VHost (Virtual Host)"),
            Mode::DNS => write!(f, "DNS (Subdomains)"),
        }
    }
}

fn extract_protocol(url: &str) -> String {
    let protocol = url.split("://").next().unwrap();
    if protocol == "https" {
        "https://".to_string()
    } else {
        "http://".to_string()
    }
}

fn clean_url(url: &str) -> String {
    let clean_url = url.replace("http://", "").replace("https://", "");
    let host = clean_url.split("/").next().unwrap();
    host.to_string()
}

fn get_url(target: &str, mode: &Mode) -> String {
    let protocol = extract_protocol(target);
    let clean_target = clean_url(target);

    match mode {
        Mode::Dir => format!("{}/{}", target, "{fuzz}"),
        Mode::Fuzz => target.to_string(),
        Mode::Vhost => target.to_string(),
        Mode::DNS => format!("{}{}.{}", protocol, "{fuzz}", clean_target),
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
    let url = get_url(&target, &mode);

    let bodies = stream::iter(wordlist)
        .map(|value| {
            let client = client.clone();
            let full_url = url.replace("{fuzz}", &value);
            let ua = random_element(uas.clone());
            let mut req = client.get(&full_url);
            let mut vhost: Option<String> = None;

            if ua.is_some() {
                req = req.header("User-Agent", ua.unwrap());
            }

            if mode == Mode::Vhost {
                let clean_url = full_url.replace("http://", "").replace("https://", "");
                let host = clean_url.split("/").next().unwrap();
                let host_with = format!("{}.{}", &value, host);
                req = req.header("Host", &host_with);
                vhost = Some(host_with);
            }

            tokio::spawn(async move {
                let resp = req.send().await?;
                let status_code = resp.status().as_u16();

                if status_code != 404 && status_code != 400 {
                    let status_display = if status_code < 200 {
                        format!("{}", status_code.to_string().cyan())
                    } else if status_code >= 200 && status_code < 300 {
                        format!("{}", status_code.to_string().green())
                    } else if status_code >= 300 && status_code < 400 {
                        format!("{}", status_code.to_string().yellow())
                    } else if status_code >= 400 && status_code < 500 {
                        format!("{}", status_code.to_string().red())
                    } else {
                        format!("{}", status_code.to_string().bright_red())
                    };

                    let info = if vhost.is_some() {
                        format!(" (VHost: {})", vhost.unwrap().bright_magenta())
                            .bright_black()
                            .to_string()
                    } else {
                        "".to_string()
                    };

                    println!(
                        "{} {} {}{} ({})",
                        ">".magenta(),
                        "Found:".cyan(),
                        full_url,
                        info,
                        status_display
                    );
                }

                resp.bytes().await
            })
        })
        .buffer_unordered(threads);

    bodies.for_each(|_| async {}).await;
}
