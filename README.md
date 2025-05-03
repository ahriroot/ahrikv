# Ahrikv (Akv)

[![Build Status](https://github.com/ahriroot/ahrikv/actions/workflows/release.yml/badge.svg)](https://github.com/ahriroot/ahrikv/actions)
[![GitHub Release](https://img.shields.io/github/v/release/ahriroot/ahrikv?style=flat-square)](https://github.com/ahriroot/ahrikv/releases)
[![License](https://img.shields.io/github/license/ahriroot/ahrikv?style=flat-square)](https://github.com/ahriroot/ahrikv)

> A high-performance in-memory key-value store.

## Usage

### Run Ahrikv Server

```bash
# run with default config
akvs

# run with config file
akvs config.toml
```

#### Default configuration

```toml
host = "127.0.0.1"
port = 60002
secret = "your_secret"
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

## License

MIT
