use clap::ValueEnum;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::{header::HeaderMap, Client};
use std::{
    fmt::{self, Write},
    sync::Arc,
};
use tokio::sync::Semaphore;

use crate::utils::random_element;

#[derive(ValueEnum, Clone, Default, Debug, PartialEq)]
pub enum Mode {
    #[default]
    Dir,
    Fuzz,
    Vhost,
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
    max_concurrent_tasks: usize,
) {
    let client = Client::builder().default_headers(headers).build().unwrap();
    let url = get_url(&target, &mode);
    let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks));

    // Create a progress bar to show the status of tasks
    let pb = ProgressBar::new(wordlist.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-"),
    );

    let tasks: Vec<_> = wordlist
        .into_iter()
        .map(|value| {
            let client = client.clone();
            let full_url = url.replace("{fuzz}", &value);
            let uas = uas.clone();
            let mode = mode.clone();
            let semaphore = semaphore.clone();
            let pb = pb.clone(); // Clone the progress bar

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                let ua = random_element(uas);
                let mut req = client.get(&full_url);
                let mut vhost: Option<String> = None;

                if let Some(ua) = ua {
                    req = req.header("User-Agent", ua);
                }

                if mode == Mode::Vhost {
                    let clean_url = full_url.replace("http://", "").replace("https://", "");
                    let host = clean_url.split("/").next().unwrap();
                    let host_with = format!("{}.{}", value, host);
                    req = req.header("Host", &host_with);
                    vhost = Some(host_with);
                }

                match req.send().await {
                    Ok(resp) => {
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

                            let info = if let Some(vhost) = vhost {
                                format!(" (VHost: {})", vhost.bright_magenta())
                                    .bright_black()
                                    .to_string()
                            } else {
                                "".to_string()
                            };

                            pb.println(format!(
                                "{} {} {}{} ({})",
                                ">".magenta(),
                                "Found:".cyan(),
                                full_url,
                                info,
                                status_display,
                            ));
                        }
                    }
                    Err(e) => {
                        pb.println(format!("{} {}: {}", "Error:".red(), "Request failed", e));
                    }
                }

                pb.inc(1); // Increment the progress bar for each task
            })
        })
        .collect();

    futures::future::join_all(tasks).await;
    pb.finish_and_clear();
    println!("{} {}", "Done".green(), "Busting completed".bright_black());
}
