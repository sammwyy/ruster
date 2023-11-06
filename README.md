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

## 📋 Usage

```bash
# Usage:
ruster [...options] target

# Directory fuzzer.
ruster -w /path/to/wordlist.txt http://example.com

# Query fuzzer.
ruster -w /path/to/wordlist.txt http://example.com/?q={value}

# Subdomain fuzzer.
ruster -w /path/to/wordlist.txt http://{value}.example.com
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
