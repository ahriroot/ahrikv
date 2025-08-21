# ahrikv (akv)

[![Build Status](https://github.com/ahriroot/ahrikv/actions/workflows/release.yml/badge.svg)](https://github.com/ahriroot/ahrikv/actions)
[![GitHub Release](https://img.shields.io/github/v/release/ahriroot/ahrikv?style=flat-square)](https://github.com/ahriroot/ahrikv/releases)
[![License](https://img.shields.io/github/license/ahriroot/ahrikv?style=flat-square)](https://github.com/ahriroot/ahrikv)

> Support for string, hash, list, set, sorted set data structures.

## Usage

### Run ahrikv Server

```bash
# run with default config
akvs

# run with config file
akvs config.toml
```

#### Default configuration

```toml
host = "127.0.0.1"     # 服务地址, 默认 127.0.0.1
port = 60002           # 端口号, 默认 60002
secret = "your_secret" # 访问密钥 secret
```

### Install by downloading binary

Download the latest binary from the [releases page](https://github.com/ahriroot/ahrikv/releases) and copy it to the desired location.

### Install from Crates.io

```bash
cargo install ahrikv
```

### Install from Source

```bash
git clone https://github.com/ahriroot/ahrikv.git
cd ahrikv
cargo build --release
```

## Features

- String
- Hash
- List
- Set
- Sorted Set

## License

MIT
