use akv::{
    command::{Authenticate, CmdType, Pong, ResultAuthenticate, ResultError},
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
        let mut head = [0u8; 12]; // 魔数(2) + 版本(1) + 类型(1) + 序列号(4) + 长度(4) = 12字节
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
                    0x06 => CmdType::Dbs,
                    0x11 => CmdType::GetString,
                    0x12 => CmdType::SetString,
                    0x13 => CmdType::DelString,
                    0x21 => CmdType::HashSet,
                    0x22 => CmdType::HashGet,
                    0x23 => CmdType::HashDel,
                    0x24 => CmdType::HashExists,
                    0x25 => CmdType::HashLen,
                    0x26 => CmdType::HashFields,
                    _ => {
                        return;
                    }
                };

                // 序列号
                let sequence = u32::from_be_bytes(head[4..8].try_into().unwrap());

                // 解析大端序长度
                let body_len = u32::from_be_bytes(head[8..12].try_into().unwrap());
                (version, req_type, sequence, body_len as usize)
            }
            Err(_) => {
                return;
            }
        };
        if header.1 != CmdType::Authenticate {
            let auth_result = ResultAuthenticate::new(false, "unauthorized".to_string());
            let _ = tx.send(auth_result.to_result(header.2).unwrap()).await;
            continue;
        }
        let mut body = vec![0u8; header.3];
        reader.read_exact(&mut body).await.unwrap();

        let auth: Authenticate = serde_json::from_slice(&body).unwrap();
        if auth.secret == state.config.secret {
            let auth_result = ResultAuthenticate::new(true, "ok".to_string());
            let _ = tx.send(auth_result.to_result(header.2).unwrap()).await;
            break;
        } else {
            let auth_result = ResultAuthenticate::new(false, "secret error".to_string());
            let _ = tx.send(auth_result.to_result(header.2).unwrap()).await;
            continue;
        }
    }

    loop {
        // 读取消息头 (8字节)
        let mut head = [0u8; 12]; // 魔数(2) + 版本(1) + 类型(1) + 序列号(4) + 长度(4) = 12字节
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
                    0x06 => CmdType::Dbs,
                    0x11 => CmdType::SetString,
                    0x12 => CmdType::GetString,
                    0x13 => CmdType::DelString,
                    0x21 => CmdType::HashSet,
                    0x22 => CmdType::HashGet,
                    0x23 => CmdType::HashDel,
                    0x24 => CmdType::HashExists,
                    0x25 => CmdType::HashLen,
                    0x26 => CmdType::HashFields,
                    _ => {
                        continue;
                    }
                };

                // 序列号
                let sequence = u32::from_be_bytes(head[4..8].try_into().unwrap());

                // 解析大端序长度
                let body_len = u32::from_be_bytes(head[8..12].try_into().unwrap());
                (version, req_type, sequence, body_len as usize)
            }
            Err(_) => {
                return;
            }
        };
        // 读取消息体 (len)
        let mut body = vec![0u8; header.3];

        match reader.read_exact(&mut body).await {
            Ok(0) => {
                break;
            }
            Ok(_) => match header.1 {
                CmdType::Ping => {
                    let _ = tx.send(Pong::default().to_result(header.2).unwrap()).await;
                }
                CmdType::Authenticate => {
                    continue;
                }
                CmdType::Keys => match state.keys(body).await {
                    Ok(keys) => {
                        let _ = tx.send(keys.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::Keys as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::Exists => match state.exists(body).await {
                    Ok(exists) => {
                        let _ = tx.send(exists.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::Exists as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::Expire => match state.expire(body).await {
                    Ok(expire) => {
                        let _ = tx.send(expire.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::Expire as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::Dbs => match state.dbs(body).await {
                    Ok(dbs) => {
                        let _ = tx.send(dbs.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::Dbs as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::SetString => match state.set_string(body).await {
                    Ok(string) => {
                        let _ = tx.send(string.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::SetString as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::GetString => match state.get_string(body).await {
                    Ok(string) => {
                        let _ = tx.send(string.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::GetString as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::DelString => match state.del_string(body).await {
                    Ok(string) => {
                        let _ = tx.send(string.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::DelString as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::HashSet => match state.hash_set(body).await {
                    Ok(hash) => {
                        let _ = tx.send(hash.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::HashSet as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::HashGet => match state.hash_get(body).await {
                    Ok(hash) => {
                        let _ = tx.send(hash.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::HashGet as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::HashDel => match state.hash_del(body).await {
                    Ok(hash) => {
                        let _ = tx.send(hash.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::HashDel as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::HashExists => match state.hash_exists(body).await {
                    Ok(hash) => {
                        let _ = tx.send(hash.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::HashExists as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::HashLen => match state.hash_len(body).await {
                    Ok(hash) => {
                        let _ = tx.send(hash.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::HashLen as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::HashFields => match state.hash_fields(body).await {
                    Ok(hash) => {
                        let _ = tx.send(hash.to_result(header.2).unwrap()).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(
                                ResultError::new(CmdType::HashFields as u8, e.to_string())
                                    .to_result(header.2)
                                    .unwrap(),
                            )
                            .await;
                    }
                },
                CmdType::Error => {}
            },
            Err(_) => {
                break;
            }
        }
    }
}
