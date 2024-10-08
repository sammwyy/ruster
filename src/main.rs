use buster::{start_buster, Mode};
use clap::Parser;
use colored::Colorize;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
};
use tokio;

mod buster;
mod utils;

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
    command: Mode,
    target: String,

    #[arg(
        short,
        long,
        help = "Extensions file to use, it will append the extension to the wordlist"
    )]
    extensions: Option<String>,

    #[arg(short = 'x', long, help = "Add headers to the request")]
    headers: Option<Vec<String>>,

    #[arg(
        short = 's',
        long,
        help = "Treat wordlist as subdomains and append target domain"
    )]
    subdomains: bool,

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

fn get_wordlist_with_ext(
    file_path: String,
    extension_path: Option<String>,
    subdomains: bool,
    target: &str,
) -> Vec<String> {
    let wordlist = get_wordlist(file_path);

    let mut wordlist_with_ext = Vec::new();

    if let Some(extension_path) = extension_path {
        let file = File::open(extension_path).unwrap();
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.unwrap();

            if line.is_empty() || line.starts_with("#") {
                continue;
            }

            for word in &wordlist {
                let mut word_with_ext = line.replace("%", word);

                if subdomains {
                    word_with_ext = format!("{}.{}", word_with_ext, target);
                }

                wordlist_with_ext.push(word_with_ext);
            }
        }
    } else {
        for word in wordlist {
            let word = if subdomains {
                format!("{}.{}", word, target)
            } else {
                word
            };

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

    // Print the banner
    println!(
        "{} (v{}) {}",
        "Ruster".magenta(),
        VERSION,
        "Directory buster tool".bright_black()
    );

    // Print the settings
    debug_setting("Mode", &args.command);
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

    // Check arguments inconsistencies
    let is_accept_ext = args.command == Mode::Dir || args.command == Mode::Fuzz;
    if !is_accept_ext && args.extensions.is_some() {
        println!(
            "{} {}",
            "Error:".red(),
            "Extensions are not supported in VHost mode".bright_black()
        );
        return;
    }

    // Start the buster
    let mode = args.command;
    let target = args.target;
    let headers = parse_headers(args.headers);
    let wordlist = get_wordlist_with_ext(args.wordlist, args.extensions, args.subdomains, &target);
    let uas: Vec<String> = parse_uas(args.user_agents);
    let threads = args.threads;

    start_buster(mode, target, headers, wordlist, uas, threads).await;
}
