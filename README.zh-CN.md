# Ahrikv (Akv) kv数据库

[![Build Status](https://github.com/ahriroot/ahrikv/actions/workflows/release.yml/badge.svg)](https://github.com/ahriroot/ahrikv/actions)
[![GitHub Release](https://img.shields.io/github/v/release/ahriroot/ahrikv?style=flat-square)](https://github.com/ahriroot/ahrikv/releases)
[![License](https://img.shields.io/github/license/ahriroot/ahrikv?style=flat-square)](https://github.com/ahriroot/ahrikv)

> 基于内存的 key-value 高性能缓存数据库。

## 使用

### 运行 Ahrikv 服务

```bash
# 默认配置运行
akvs

# 指定配置文件运行
akvs config.toml
```

#### 默认配置

```toml
host = "127.0.0.1"     # 服务地址
port = 60002           # 端口号
secret = "your_secret" # 访问密钥
```

### 下载可执行文件

从 [发布页面](https://github.com/ahriroot/ahrikv/releases) 下载最新可执行文件，并将其复制到所需位置。

### 从 Crates.io 安装

```bash
cargo install ahrikv
```

### 从源码安装

```bash
git clone https://github.com/ahriroot/ahrikv.git
cd ahrikv
cargo build --release
```

## 许可证

MIT
