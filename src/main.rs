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

    #[arg(
        short,
        long,
        help = "Extensions file to use, it will append the extension to the wordlist"
    )]
    extensions: Option<String>,

    #[arg(short = 'x', long, help = "Add headers to the request")]
    headers: Option<Vec<String>>,

    #[arg(short, long, default_value_t = 4, help = "Number of threads")]
    threads: usize,

    #[arg(
        short,
        long,
        help = "User agents file to use, it will randomize the user agent"
    )]
    user_agents: Option<String>,

    #[arg(short, long, help = "Wordlist file to use")]
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

fn get_wordlist_with_ext(file_path: String, extension_path: Option<String>) -> Vec<String> {
    let wordlist = get_wordlist(file_path);

    if extension_path.is_none() {
        return wordlist;
    }

    let mut wordlist_with_ext: Vec<String> = Vec::new();
    let extension_path = extension_path.unwrap();
    let file = File::open(extension_path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();

        if line.is_empty() || line.starts_with("#") {
            continue;
        }

        for word in &wordlist {
            let word = line.replace("%", word);
            wordlist_with_ext.push(word);
        }
    }

    wordlist_with_ext
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

fn parse_uas(file: Option<String>) -> Vec<String> {
    if file.is_none() {
        return Vec::new();
    }

    let file = file.unwrap();
    let file = File::open(file).unwrap();
    let reader = BufReader::new(file);
    let mut uas = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap();

        if line.is_empty() || line.starts_with("#") {
            continue;
        }

        uas.push(line);
    }

    uas
}

fn random_element(vec: Vec<String>) -> Option<String> {
    if vec.is_empty() {
        return None;
    }

    let random_index = rand::random::<usize>() % vec.len();
    let element = vec[random_index].clone();
    Some(element)
}

async fn start_buster(arg: Args) {
    let headers = parse_headers(arg.headers);
    let target = fix_target(arg.target);
    let wordlist = get_wordlist_with_ext(arg.wordlist, arg.extensions);
    let threads = arg.threads;
    let uas: Vec<String> = parse_uas(arg.user_agents);

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

    if args.extensions.is_some() {
        let extensions = args.extensions.clone().unwrap();
        debug_setting("Extensions", &extensions);
    }

    if args.headers.is_some() {
        let headers = args.headers.clone().unwrap();
        debug_settings_list("Headers", &headers);
    }

    debug_setting("Threads", &args.threads);
    debug_setting("Wordlist", &args.wordlist);

    if args.user_agents.is_some() {
        let user_agents = args.user_agents.clone().unwrap();
        debug_setting("User agents", &user_agents);
    }

    start_buster(args).await;
}
