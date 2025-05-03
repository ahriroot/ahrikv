use akv::{
    command::{Authenticate, CmdType, Pong, ResultAuthenticate},
    MAGIC_NUMBER,
};
use tokio::{
    io::{self, AsyncReadExt as _, AsyncWriteExt as _},
    net::TcpStream,
};

use crate::server::state::State;

pub async fn handler(socket: TcpStream, mut state: State) {
    let (mut reader, mut writer) = io::split(socket);
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(128);

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            let _ = writer.write_all(&message).await;
        }
    });

    loop {
        // 读取消息头 (8字节)
        let mut head = [0u8; 8]; // 魔数(2) + 版本(1) + 类型(1) + 长度(4) = 8字节
        let header = match reader.read_exact(&mut head).await {
            Ok(_) => {
                if head[0..2] != MAGIC_NUMBER {
                    return;
                }
                // 解析版本
                let version = head[2];

                // 解析请求类型
                let req_type = match head[3] {
                    0x01 => CmdType::Ping,
                    0x02 => CmdType::Authenticate,
                    0x03 => CmdType::Keys,
                    0x04 => CmdType::Exists,
                    0x05 => CmdType::Expire,
                    0x11 => CmdType::GetString,
                    0x12 => CmdType::SetString,
                    0x13 => CmdType::DelString,
                    _ => {
                        return;
                    }
                };

                // 解析大端序长度
                let body_len = u32::from_be_bytes(head[4..8].try_into().unwrap());
                (version, req_type, body_len as usize)
            }
            Err(_) => {
                return;
            }
        };
        if header.1 != CmdType::Authenticate {
            let auth_result = ResultAuthenticate::new(false, "unauthorized".to_string());
            let _ = tx.send(auth_result.to_result().unwrap()).await;
            continue;
        }
        let mut body = vec![0u8; header.2];
        reader.read_exact(&mut body).await.unwrap();

        let auth: Authenticate = serde_json::from_slice(&body).unwrap();
        if auth.secret == state.config.secret {
            let auth_result = ResultAuthenticate::new(true, "ok".to_string());
            let _ = tx.send(auth_result.to_result().unwrap()).await;
            break;
        } else {
            let auth_result = ResultAuthenticate::new(false, "secret error".to_string());
            let _ = tx.send(auth_result.to_result().unwrap()).await;
            continue;
        }
    }

    loop {
        // 读取消息头 (8字节)
        let mut head = [0u8; 8]; // 魔数(2) + 版本(1) + 类型(1) + 长度(4) = 8字节
        let header = match reader.read_exact(&mut head).await {
            Ok(_) => {
                if head[0..2] != MAGIC_NUMBER {
                    return;
                }
                // 解析版本
                let version = head[2];

                // 解析请求类型
                let req_type = match head[3] {
                    0x01 => CmdType::Ping,
                    0x02 => CmdType::Authenticate,
                    0x03 => CmdType::Keys,
                    0x04 => CmdType::Exists,
                    0x05 => CmdType::Expire,
                    0x11 => CmdType::SetString,
                    0x12 => CmdType::GetString,
                    0x13 => CmdType::DelString,
                    _ => {
                        continue;
                    }
                };

                // 解析大端序长度
                let body_len = u32::from_be_bytes(head[4..8].try_into().unwrap());
                (version, req_type, body_len as usize)
            }
            Err(_) => {
                return;
            }
        };
        // 读取消息体 (len)
        let mut body = vec![0u8; header.2];
        match reader.read_exact(&mut body).await {
            Ok(0) => {
                break;
            }
            Ok(_) => match header.1 {
                CmdType::Ping => {
                    let _ = tx.send(Pong::default().to_result().unwrap()).await;
                }
                CmdType::Authenticate => {
                    continue;
                }
                CmdType::Keys => {
                    let keys = state.keys(body).await.unwrap();
                    let _ = tx.send(keys.to_result().unwrap()).await;
                }
                CmdType::Exists => {
                    let exists = state.exists(body).await.unwrap();
                    let _ = tx.send(exists.to_result().unwrap()).await;
                }
                CmdType::Expire => {
                    let expire = state.expire(body).await.unwrap();
                    let _ = tx.send(expire.to_result().unwrap()).await;
                }
                CmdType::SetString => {
                    let string = state.set_string(body).await.unwrap();
                    let _ = tx.send(string.to_result().unwrap()).await;
                }
                CmdType::GetString => {
                    let string = state.get_string(body).await.unwrap();
                    let _ = tx.send(string.to_result().unwrap()).await;
                }
                CmdType::DelString => {
                    let string = state.del_string(body).await.unwrap();
                    let _ = tx.send(string.to_result().unwrap()).await;
                }
            },
            Err(_) => {
                break;
            }
        }
    }
}
