use clap::Parser;
use rustyline::{error::ReadlineError, history::History, DefaultEditor};
use std::{env, error::Error, fs, path::Path};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use akv::{
    command::{
        hash::{
            HashDel, HashExists, HashFields, HashGet, HashLen, HashSet, ResultHashDel,
            ResultHashExists, ResultHashFields, ResultHashGet, ResultHashLen, ResultHashSet,
        },
        string::{
            DelString, GetString, ResultDelString, ResultGetString, ResultSetString, SetString,
        },
        Authenticate, CmdType, Exists, Expire, Keys, ResultAuthenticate, ResultExists,
        ResultExpire, ResultKeys,
    },
    config::Config,
    MAGIC_NUMBER, VERSION,
};

/// Ahrikv CLI client
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to config file
    #[arg(short, long, value_name = "FILE", default_value = "./config.toml")]
    config: String,

    /// Server host
    #[arg(short = 'H', long, value_name = "HOST")]
    host: Option<String>,

    /// Server port
    #[arg(short, long, value_name = "PORT")]
    port: Option<u16>,

    /// Authentication secret
    #[arg(short, long, value_name = "SECRET")]
    secret: Option<String>,
}

struct Client {
    stream: TcpStream,
    seq: u32,
}

impl Client {
    async fn new(addr: &str, secret: &str) -> Result<Self, Box<dyn Error>> {
        let stream = TcpStream::connect(addr).await?;
        let mut client = Self { stream, seq: 0 };

        client.authenticate(secret).await?;
        Ok(client)
    }

    async fn authenticate(&mut self, secret: &str) -> Result<(), Box<dyn Error>> {
        let auth = Authenticate {
            secret: secret.to_string(),
        };
        let data = serde_json::to_vec(&auth)?;
        let data_length = data.len() as u32;

        let mut request = vec![];
        request.extend(&MAGIC_NUMBER);
        request.push(VERSION);
        request.push(CmdType::Authenticate as u8);
        request.extend(&self.seq.to_be_bytes());
        self.seq += 1;
        request.extend(&data_length.to_be_bytes());
        request.extend(data);

        self.stream.write_all(&request).await?;

        let mut head = [0u8; 12];
        self.stream.read_exact(&mut head).await?;

        if head[0..2] != MAGIC_NUMBER {
            return Err("Invalid magic number".into());
        }

        let body_len = u32::from_be_bytes(head[8..12].try_into()?) as usize;
        let mut body = vec![0u8; body_len];
        self.stream.read_exact(&mut body).await?;

        let result: ResultAuthenticate = serde_json::from_slice(&body)?;
        if !result.ok {
            return Err(format!("Authentication failed: {}", result.msg).into());
        }

        Ok(())
    }

    async fn send_request(
        &mut self,
        cmd_type: CmdType,
        data: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let data_length = data.len() as u32;

        let mut request = vec![];
        request.extend(&MAGIC_NUMBER);
        request.push(VERSION);
        request.push(cmd_type as u8);
        request.extend(&self.seq.to_be_bytes());
        self.seq += 1;
        request.extend(&data_length.to_be_bytes());
        request.extend(data);

        self.stream.write_all(&request).await?;

        let mut head = [0u8; 12];
        self.stream.read_exact(&mut head).await?;

        if head[0..2] != MAGIC_NUMBER {
            return Err("Invalid magic number".into());
        }

        let body_len = u32::from_be_bytes(head[8..12].try_into()?) as usize;
        let mut body = vec![0u8; body_len];
        self.stream.read_exact(&mut body).await?;

        Ok(body)
    }

    async fn set(
        &mut self,
        db: &str,
        key: &str,
        value: &str,
        expire: Option<u64>,
    ) -> Result<(), Box<dyn Error>> {
        let cmd = SetString {
            db: db.to_string(),
            key: key.to_string(),
            value: value.to_string(),
            expire,
        };
        let data = serde_json::to_vec(&cmd)?;
        let body = self.send_request(CmdType::SetString, data).await?;
        let result: ResultSetString = serde_json::from_slice(&body)?;
        if !result.ok {
            return Err(result.msg.into());
        }
        Ok(())
    }

    async fn get(&mut self, db: &str, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        let cmd = GetString {
            db: db.to_string(),
            key: key.to_string(),
        };
        let data = serde_json::to_vec(&cmd)?;
        let body = self.send_request(CmdType::GetString, data).await?;
        let result: ResultGetString = serde_json::from_slice(&body)?;
        Ok(result.value)
    }

    async fn del(&mut self, db: &str, key: &str) -> Result<Option<String>, Box<dyn Error>> {
        let cmd = DelString {
            db: db.to_string(),
            key: key.to_string(),
        };
        let data = serde_json::to_vec(&cmd)?;
        let body = self.send_request(CmdType::DelString, data).await?;
        let result: ResultDelString = serde_json::from_slice(&body)?;
        Ok(result.value)
    }

    async fn exists(&mut self, db: &str, key: &str) -> Result<bool, Box<dyn Error>> {
        let cmd = Exists {
            db: db.to_string(),
            key: key.to_string(),
        };
        let data = serde_json::to_vec(&cmd)?;
        let body = self.send_request(CmdType::Exists, data).await?;
        let result: ResultExists = serde_json::from_slice(&body)?;
        Ok(result.exists)
    }

    async fn expire(&mut self, db: &str, key: &str, expire: u64) -> Result<bool, Box<dyn Error>> {
        let cmd = Expire {
            db: db.to_string(),
            key: key.to_string(),
            expire,
        };
        let data = serde_json::to_vec(&cmd)?;
        let body = self.send_request(CmdType::Expire, data).await?;
        let result: ResultExpire = serde_json::from_slice(&body)?;
        Ok(result.ok)
    }

    async fn keys(
        &mut self,
        db: &str,
        page: usize,
        size: usize,
    ) -> Result<(Vec<String>, u32), Box<dyn Error>> {
        let cmd = Keys {
            db: db.to_string(),
            page,
            size,
        };
        let data = serde_json::to_vec(&cmd)?;
        let body = self.send_request(CmdType::Keys, data).await?;
        let result: ResultKeys = serde_json::from_slice(&body)?;
        Ok((result.keys, result.total))
    }

    async fn hset(
        &mut self,
        db: &str,
        key: &str,
        field: &str,
        value: &str,
        expire: Option<u64>,
    ) -> Result<(), Box<dyn Error>> {
        let cmd = HashSet {
            db: db.to_string(),
            key: key.to_string(),
            field: field.to_string(),
            value: value.to_string(),
            expire,
        };
        let data = serde_json::to_vec(&cmd)?;
        let body = self.send_request(CmdType::HashSet, data).await?;
        let result: ResultHashSet = serde_json::from_slice(&body)?;
        if !result.ok {
            return Err(result.msg.into());
        }
        Ok(())
    }

    async fn hget(
        &mut self,
        db: &str,
        key: &str,
        field: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let cmd = HashGet {
            db: db.to_string(),
            key: key.to_string(),
            field: field.to_string(),
        };
        let data = serde_json::to_vec(&cmd)?;
        let body = self.send_request(CmdType::HashGet, data).await?;
        let result: ResultHashGet = serde_json::from_slice(&body)?;
        Ok(result.value)
    }

    async fn hdel(
        &mut self,
        db: &str,
        key: &str,
        field: &str,
    ) -> Result<Option<String>, Box<dyn Error>> {
        let cmd = HashDel {
            db: db.to_string(),
            key: key.to_string(),
            field: field.to_string(),
        };
        let data = serde_json::to_vec(&cmd)?;
        let body = self.send_request(CmdType::HashDel, data).await?;
        let result: ResultHashDel = serde_json::from_slice(&body)?;
        Ok(result.value)
    }

    async fn hexists(&mut self, db: &str, key: &str, field: &str) -> Result<bool, Box<dyn Error>> {
        let cmd = HashExists {
            db: db.to_string(),
            key: key.to_string(),
            field: field.to_string(),
        };
        let data = serde_json::to_vec(&cmd)?;
        let body = self.send_request(CmdType::HashExists, data).await?;
        let result: ResultHashExists = serde_json::from_slice(&body)?;
        Ok(result.exists)
    }

    async fn hlen(&mut self, db: &str, key: &str) -> Result<u32, Box<dyn Error>> {
        let cmd = HashLen {
            db: db.to_string(),
            key: key.to_string(),
        };
        let data = serde_json::to_vec(&cmd)?;
        let body = self.send_request(CmdType::HashLen, data).await?;
        let result: ResultHashLen = serde_json::from_slice(&body)?;
        Ok(result.len)
    }

    async fn hfields(&mut self, db: &str, key: &str) -> Result<(Vec<String>, u32), Box<dyn Error>> {
        let cmd = HashFields {
            db: db.to_string(),
            key: key.to_string(),
        };
        let data = serde_json::to_vec(&cmd)?;
        let body = self.send_request(CmdType::HashFields, data).await?;
        let result: ResultHashFields = serde_json::from_slice(&body)?;
        Ok((result.fields, result.total))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let config = Config::new(Some(&cli.config))?;
    let addr = config.get_address();
    let secret = &config.secret;

    let mut client = Client::new(&addr, secret).await?;
    println!("Connected to {}", addr);
    println!("Type 'help' for available commands, 'quit' to exit");

    let mut db = "default".to_string();

    // 创建rustyline编辑器
    let mut rl = DefaultEditor::new()?;

    // 获取用户目录并构建历史文件路径
    let history_path = match env::var("HOME") {
        Ok(home) => {
            // Unix系统
            let ahrikv_dir = Path::new(&home).join(".ahriknow").join("ahrikv");
            // 确保目录存在
            if let Err(_) = fs::create_dir_all(&ahrikv_dir) {
                // 如果创建目录失败，使用当前目录
                Path::new("akvc_history.txt").to_path_buf()
            } else {
                ahrikv_dir.join("akvc_history.txt")
            }
        }
        Err(_) => match env::var("USERPROFILE") {
            Ok(userprofile) => {
                // Windows系统
                let ahrikv_dir = Path::new(&userprofile).join(".ahriknow").join("ahrikv");
                // 确保目录存在
                if let Err(_) = fs::create_dir_all(&ahrikv_dir) {
                    // 如果创建目录失败，使用当前目录
                    Path::new("akvc_history.txt").to_path_buf()
                } else {
                    ahrikv_dir.join("akvc_history.txt")
                }
            }
            Err(_) => {
                // 如果无法获取用户目录，使用当前目录
                Path::new("akvc_history.txt").to_path_buf()
            }
        },
    };

    // 设置历史记录最大长度为1000条
    let _ = rl.history_mut().set_max_len(1000);

    // 加载历史记录
    let _ = rl.load_history(&history_path);

    loop {
        let prompt = format!("{}> ", db);
        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                // 添加到历史记录
                let _ = rl.add_history_entry(line.as_str());

                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                if input == "quit" || input == "exit" {
                    println!("Goodbye!");
                    break;
                }

                if input == "help" {
                    print_help();
                    continue;
                }

                let parts = parse_input(input);
                if parts.is_empty() {
                    continue;
                }

                let cmd = parts[0].to_lowercase();
                let result = match cmd.as_str() {
                    "set" => handle_set(&mut client, &mut db, parts.as_slice()).await,
                    "get" => handle_get(&mut client, &db, parts.as_slice()).await,
                    "del" => handle_del(&mut client, &db, parts.as_slice()).await,
                    "exists" => handle_exists(&mut client, &db, parts.as_slice()).await,
                    "expire" => handle_expire(&mut client, &db, parts.as_slice()).await,
                    "keys" => handle_keys(&mut client, &db, parts.as_slice()).await,
                    "use" => handle_use(&mut db, parts.as_slice()),
                    "hset" => handle_hset(&mut client, &db, parts.as_slice()).await,
                    "hget" => handle_hget(&mut client, &db, parts.as_slice()).await,
                    "hdel" => handle_hdel(&mut client, &db, parts.as_slice()).await,
                    "hexists" => handle_hexists(&mut client, &db, parts.as_slice()).await,
                    "hlen" => handle_hlen(&mut client, &db, parts.as_slice()).await,
                    "hfields" => handle_hfields(&mut client, &db, parts.as_slice()).await,
                    _ => Err(format!(
                        "Unknown command: {}. Type 'help' for available commands.",
                        cmd
                    )
                    .into()),
                };

                match result {
                    Ok(msg) => {
                        if !msg.is_empty() {
                            println!("{}", msg);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("\n^C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("\nBye!");
                break;
            }
            Err(err) => {
                eprintln!("Failed to read line: {:?}", err);
                break;
            }
        }
    }

    // 保存历史记录
    let _ = rl.save_history(&history_path);

    Ok(())
}

fn print_help() {
    print!(
        r#"Available commands:
    SET <key> <value> [expire]           - Set a string value
    GET <key>                            - Get a string value
    DEL <key>                            - Delete a key
    EXISTS <key>                         - Check if a key exists
    EXPIRE <key> <seconds>               - Set expiration time
    KEYS [page] [size]                   - List all keys
    USE <db>                             - Switch database
    HSET <key> <field> <value> [expire]  - Set hash field
    HGET <key> <field>                   - Get hash field
    HDEL <key> <field>                   - Delete hash field
    HEXISTS <key> <field>                - Check if hash field exists
    HLEN <key>                           - Get hash length
    HFIELDS <key>                        - Get all hash fields
    HELP                                 - Show this help
    QUIT/EXIT                            - Exit the CLI
"#
    );
}

fn parse_input(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' {
            escape = true;
            continue;
        }

        if ch == '"' {
            in_quotes = !in_quotes;
            continue;
        }

        if ch.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
            continue;
        }

        current.push(ch);
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

async fn handle_set(
    client: &mut Client,
    db: &str,
    parts: &[String],
) -> Result<String, Box<dyn Error>> {
    if parts.len() < 3 {
        return Err("Usage: SET <key> <value> [expire]".into());
    }
    let key = &parts[1];
    let value = &parts[2];
    let expire = if parts.len() > 3 {
        Some(parts[3].parse()?)
    } else {
        None
    };
    client.set(db, key, value, expire).await?;
    Ok("OK".to_string())
}

async fn handle_get(
    client: &mut Client,
    db: &str,
    parts: &[String],
) -> Result<String, Box<dyn Error>> {
    if parts.len() < 2 {
        return Err("Usage: GET <key>".into());
    }
    let key = &parts[1];
    match client.get(db, key).await? {
        Some(value) => Ok(format!("\"{}\"", value)),
        None => Ok("(nil)".to_string()),
    }
}

async fn handle_del(
    client: &mut Client,
    db: &str,
    parts: &[String],
) -> Result<String, Box<dyn Error>> {
    if parts.len() < 2 {
        return Err("Usage: DEL <key>".into());
    }
    let key = &parts[1];
    match client.del(db, key).await? {
        Some(value) => Ok(format!("\"{}\"", value)),
        None => Ok("(nil)".to_string()),
    }
}

async fn handle_exists(
    client: &mut Client,
    db: &str,
    parts: &[String],
) -> Result<String, Box<dyn Error>> {
    if parts.len() < 2 {
        return Err("Usage: EXISTS <key>".into());
    }
    let key = &parts[1];
    let exists = client.exists(db, key).await?;
    Ok(if exists { "1" } else { "0" }.to_string())
}

async fn handle_expire(
    client: &mut Client,
    db: &str,
    parts: &[String],
) -> Result<String, Box<dyn Error>> {
    if parts.len() < 3 {
        return Err("Usage: EXPIRE <key> <seconds>".into());
    }
    let key = &parts[1];
    let expire: u64 = parts[2].parse()?;
    let ok = client.expire(db, key, expire).await?;
    Ok(if ok { "1" } else { "0" }.to_string())
}

async fn handle_keys(
    client: &mut Client,
    db: &str,
    parts: &[String],
) -> Result<String, Box<dyn Error>> {
    let page = if parts.len() > 1 {
        parts[1].parse().unwrap_or(1)
    } else {
        1
    };
    let size = if parts.len() > 2 {
        parts[2].parse().unwrap_or(100)
    } else {
        100
    };
    let (keys, total) = client.keys(db, page, size).await?;
    let mut result = format!("Total: {}\n", total);
    for key in keys {
        result.push_str(&format!("  {}\n", key));
    }
    Ok(result)
}

fn handle_use(db: &mut String, parts: &[String]) -> Result<String, Box<dyn Error>> {
    if parts.len() < 2 {
        return Err("Usage: USE <db>".into());
    }
    *db = parts[1].clone();
    Ok(format!("Switched to database \"{}\"", db))
}

async fn handle_hset(
    client: &mut Client,
    db: &str,
    parts: &[String],
) -> Result<String, Box<dyn Error>> {
    if parts.len() < 4 {
        return Err("Usage: HSET <key> <field> <value> [expire]".into());
    }
    let key = &parts[1];
    let field = &parts[2];
    let value = &parts[3];
    let expire = if parts.len() > 4 {
        Some(parts[4].parse()?)
    } else {
        None
    };
    client.hset(db, key, field, value, expire).await?;
    Ok("OK".to_string())
}

async fn handle_hget(
    client: &mut Client,
    db: &str,
    parts: &[String],
) -> Result<String, Box<dyn Error>> {
    if parts.len() < 3 {
        return Err("Usage: HGET <key> <field>".into());
    }
    let key = &parts[1];
    let field = &parts[2];
    match client.hget(db, key, field).await? {
        Some(value) => Ok(format!("\"{}\"", value)),
        None => Ok("(nil)".to_string()),
    }
}

async fn handle_hdel(
    client: &mut Client,
    db: &str,
    parts: &[String],
) -> Result<String, Box<dyn Error>> {
    if parts.len() < 3 {
        return Err("Usage: HDEL <key> <field>".into());
    }
    let key = &parts[1];
    let field = &parts[2];
    match client.hdel(db, key, field).await? {
        Some(value) => Ok(format!("\"{}\"", value)),
        None => Ok("(nil)".to_string()),
    }
}

async fn handle_hexists(
    client: &mut Client,
    db: &str,
    parts: &[String],
) -> Result<String, Box<dyn Error>> {
    if parts.len() < 3 {
        return Err("Usage: HEXISTS <key> <field>".into());
    }
    let key = &parts[1];
    let field = &parts[2];
    let exists = client.hexists(db, key, field).await?;
    Ok(if exists { "1" } else { "0" }.to_string())
}

async fn handle_hlen(
    client: &mut Client,
    db: &str,
    parts: &[String],
) -> Result<String, Box<dyn Error>> {
    if parts.len() < 2 {
        return Err("Usage: HLEN <key>".into());
    }
    let key = &parts[1];
    let len = client.hlen(db, key).await?;
    Ok(len.to_string())
}

async fn handle_hfields(
    client: &mut Client,
    db: &str,
    parts: &[String],
) -> Result<String, Box<dyn Error>> {
    if parts.len() < 2 {
        return Err("Usage: HFIELDS <key>".into());
    }
    let key = &parts[1];
    let (fields, total) = client.hfields(db, key).await?;
    let mut result = format!("Total: {}\n", total);
    for field in fields {
        result.push_str(&format!("  {}\n", field));
    }
    Ok(result)
}
