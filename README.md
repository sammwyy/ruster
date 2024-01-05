# 🦀 Ruster

Directory buster tool written in Rust

## ⚡ Features

- [X] Directory fuzzing
- [X] Query fuzzing
- [X] Subdomain fuzzing
- [X] Extensions
- [X] Randomize User agents
- [X] Customize Headers
- [X] Multithreading and parallelism
- [X] Multiple modes

## 📋 Usage

```bash
# Available modes:
#   dir     -   Search for files or directories in the target
#   fuzz    -   Fuzz the target with the wordlist, replacing {fuzz} with the word
#   vhost   -   Search for Virtual-Hosts in the target
#   dns     -   Search for Subdomains in the target

# Usage:
ruster <mode> [...options] <target>

# Directory fuzzer.
ruster dir -w /path/to/wordlist.txt http://example.com

# File fuzzer
ruster dir -w /path/to/wordlist.txt -e /path/to/extensions.txt http://example.com/

# Query fuzzer.
ruster dir -w /path/to/wordlist.txt http://example.com/?q={fuzz}

# Subdomain fuzzer.
ruster dns -w /path/to/wordlist.txt https://example.com

# Virtual host fuzzer.
ruster vhost -w /path/to/wordlist.txt http://example.com
```

## 📗 Arguments

| Argument | Description | Type | Default | Required |
| --- | --- | --- | --- | --- |
| -e, --extensions | Extensions file to append to wordlist | File | None | ❌ |
| -x, --header | Header to send with request | List(String) | None | ❌ |
| -w, --wordlist | Path to wordlist | File | None | ✅ |
| -t, --threads | Number of threads | Integer | 4 | ❌ |
| -u, --user_agent | File with user agents | File | None | ❌ |

## 📦 Build

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/sammwyy/ruster

# Build
cd ruster && cargo build --release
```

## 🤝 Contributing

Contributions, issues and feature requests are welcome! Feel free to check [issues page](https://github.com/sammwyy/ruster/issues).

## ❤️ Show your support

Give a ⭐️ if this project helped you! Or buy me a coffeelatte 🙌 on [Ko-fi](https://ko-fi.com/sammwy)

## 📝 License

Copyright © 2023 [Sammwy](https://github.com/sammwyy). This project is [MIT](LICENSE) licensed.
