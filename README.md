# Ruster

Directory buster tool written in Rust

## Usage

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

## Arguments

| Argument | Description | Type | Default | Required |
| --- | --- | --- | --- | --- |
| -x, --header | Header to send with request | List(String) | None | ❌ |
| -w, --wordlist | Path to wordlist | String | None | ✅ |
| -t, --threads | Number of threads | Integer | 4 | ❌ |
