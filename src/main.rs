use clap::Parser;
use colored::Colorize;
use futures::{stream, StreamExt}; // 0.3.27
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
}; // 0.11.14
use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
};
use tokio; // 1.26.0, features = ["macros"]

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
#[command(
    name = "ruster",
    version = VERSION,
    author = "Sammwy",
    about = "Directory Buster",
    long_about = "Directory buster tool written in Rust"
)]
struct Args {
    target: String,

    #[arg(short = 'x', long)]
    headers: Option<Vec<String>>,

    #[arg(short, long, default_value_t = 4)]
    threads: usize,

    #[arg(short, long)]
    wordlist: String,
}

fn fix_target(target: String) -> String {
    let target = if target.ends_with('/') {
        target
    } else {
        format!("{}/", target)
    };
    let target = if target.contains("{value}") {
        target
    } else {
        format!("{}{{value}}", target)
    };
    target
}

fn get_wordlist(file_path: String) -> Vec<String> {
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let mut wordlist = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap();

        if line.is_empty() || line.starts_with("#") {
            continue;
        }

        wordlist.push(line);
    }

    wordlist
}

fn parse_headers(list: Option<Vec<String>>) -> HeaderMap {
    let mut headers = HeaderMap::new();

    if list.is_none() {
        return headers;
    }

    let list = list.unwrap();
    for header in list {
        let header: Vec<String> = header.split(":").map(|s| s.to_string()).collect();

        if header.len() != 2 {
            continue;
        }

        let key = header[0].trim().to_lowercase();
        let value = header[1].trim().to_lowercase();

        let key_header: HeaderName = key.parse().unwrap();
        let key_value: HeaderValue = value.parse().unwrap();

        headers.entry(key_header).or_insert(key_value);
    }

    headers
}

async fn start_buster(arg: Args) {
    let headers = parse_headers(arg.headers);
    let target = fix_target(arg.target);
    let wordlist = get_wordlist(arg.wordlist);
    let threads = arg.threads;

    let client = Client::builder().default_headers(headers).build().unwrap();

    let bodies = stream::iter(wordlist)
        .map(|value| {
            let client = client.clone();
            let full_url = target.replace("{value}", &value);

            tokio::spawn(async move {
                let resp = client.get(&full_url).send().await?;
                let status_code = resp.status().as_u16();

                if status_code != 404 {
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

fn debug_setting(key: &str, value: &dyn Display) {
    let key = format!("{: <10}", key).green();
    let value = format!("{}", value);
    println!("{} {} {}", ">".magenta(), key, value);
}

fn debug_settings_list(key: &str, values: &Vec<String>) {
    let key = format!("{: <10}", key).green();
    println!("{} {}", ">".magenta(), key);

    for value in values {
        let value = format!("{}", value);
        println!("    {} {}", ">".cyan(), value.red());
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!(
        "{} (v{}) {}",
        "Ruster".magenta(),
        VERSION,
        "Directory buster tool".bright_black()
    );
    debug_setting("Target", &args.target);
    if args.headers.is_some() {
        let headers = args.headers.clone().unwrap();
        debug_settings_list("Headers", &headers);
    }
    debug_setting("List", &args.wordlist);
    debug_setting("Threads", &args.threads);
    println!("");

    start_buster(args).await;
}
