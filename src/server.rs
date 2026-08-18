use crate::models::{NoticeMsg, SayMsg, SystemMsg, UserFlushMsg, WsEnvelope};
use chrono::Local;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub type Tx = mpsc::UnboundedSender<String>;

#[derive(Default)]
pub struct BottyHistory {
    pub status_report: Option<NoticeMsg>,
    pub found_records: Vec<NoticeMsg>,
}

#[derive(Default)]
pub struct ChatServerState {
    pub clients: HashMap<String, Tx>,
    pub users: Vec<String>,
    pub msg_history: Vec<SayMsg>,
    pub botty_history: BottyHistory,
}

#[derive(Clone)]
pub struct ChatServer {
    pub state: Arc<Mutex<ChatServerState>>,
}

impl ChatServer {
    pub fn new() -> Self {
        ChatServer {
            state: Arc::new(Mutex::new(ChatServerState::default())),
        }
    }

    pub async fn connect_user(&self, username: String, tx: Tx) {
        let mut state = self.state.lock().await;
        let is_new_user = !state.clients.contains_key(&username);

        if is_new_user {
            state.users.insert(0, username.clone());
            let online_sys = SystemMsg {
                msg_type: "online".to_string(),
                msg: username.clone(),
                time: Local::now().timestamp_millis(),
            };
            let online_env = WsEnvelope {
                event_type: "system".to_string(),
                data: serde_json::to_value(&online_sys).unwrap(),
            };
            let online_json = serde_json::to_string(&online_env).unwrap();

            let flush_env = WsEnvelope {
                event_type: "userflush".to_string(),
                data: serde_json::to_value(&UserFlushMsg {
                    users: state.users.clone(),
                })
                .unwrap(),
            };
            let flush_json = serde_json::to_string(&flush_env).unwrap();

            for (_, client_tx) in state.clients.iter() {
                let _ = client_tx.send(online_json.clone());
                let _ = client_tx.send(flush_json.clone());
            }

            let in_sys = SystemMsg {
                msg_type: "in".to_string(),
                msg: "".to_string(),
                time: Local::now().timestamp_millis(),
            };
            let in_env = WsEnvelope {
                event_type: "system".to_string(),
                data: serde_json::to_value(&in_sys).unwrap(),
            };
            let _ = tx.send(serde_json::to_string(&in_env).unwrap());
        }

        let flush_self = WsEnvelope {
            event_type: "userflush".to_string(),
            data: serde_json::to_value(&UserFlushMsg {
                users: state.users.clone(),
            })
            .unwrap(),
        };
        let _ = tx.send(serde_json::to_string(&flush_self).unwrap());

        for msg in &state.msg_history {
            let env = WsEnvelope {
                event_type: "say".to_string(),
                data: serde_json::to_value(msg).unwrap(),
            };
            let _ = tx.send(serde_json::to_string(&env).unwrap());
        }

        if let Some(status_report) = &state.botty_history.status_report {
            let env = WsEnvelope {
                event_type: "notice".to_string(),
                data: serde_json::to_value(status_report).unwrap(),
            };
            let _ = tx.send(serde_json::to_string(&env).unwrap());
        }

        for record in &state.botty_history.found_records {
            let env = WsEnvelope {
                event_type: "notice".to_string(),
                data: serde_json::to_value(record).unwrap(),
            };
            let _ = tx.send(serde_json::to_string(&env).unwrap());
        }

        state.clients.insert(username, tx);
    }

    pub async fn disconnect_user(&self, username: &str) {
        let mut state = self.state.lock().await;
        if state.clients.remove(username).is_some() {
            state.users.retain(|u| u != username);

            let offline_sys = SystemMsg {
                msg_type: "offline".to_string(),
                msg: username.to_string(),
                time: Local::now().timestamp_millis(),
            };
            let offline_env = WsEnvelope {
                event_type: "system".to_string(),
                data: serde_json::to_value(&offline_sys).unwrap(),
            };
            let offline_json = serde_json::to_string(&offline_env).unwrap();

            let flush_env = WsEnvelope {
                event_type: "userflush".to_string(),
                data: serde_json::to_value(&UserFlushMsg {
                    users: state.users.clone(),
                })
                .unwrap(),
            };
            let flush_json = serde_json::to_string(&flush_env).unwrap();

            for (_, client_tx) in state.clients.iter() {
                let _ = client_tx.send(offline_json.clone());
                let _ = client_tx.send(flush_json.clone());
            }
        }
    }

    pub async fn broadcast_say(&self, mut say_msg: SayMsg) {
        let mut state = self.state.lock().await;
        say_msg.notice = true;
        let env = WsEnvelope {
            event_type: "say".to_string(),
            data: serde_json::to_value(&say_msg).unwrap(),
        };
        let msg_json = serde_json::to_string(&env).unwrap();

        if say_msg.data.to == "all" {
            for (_, client_tx) in state.clients.iter() {
                let _ = client_tx.send(msg_json.clone());
            }
            say_msg.notice = false;
            state.msg_history.push(say_msg);
            if state.msg_history.len() > 30 {
                state.msg_history.remove(0);
            }
        } else {
            if let Some(target_tx) = state.clients.get(&say_msg.data.to) {
                let _ = target_tx.send(msg_json.clone());
            }
            if say_msg.data.from != say_msg.data.to {
                if let Some(sender_tx) = state.clients.get(&say_msg.data.from) {
                    let _ = sender_tx.send(msg_json);
                }
            }
        }
    }

    pub async fn broadcast_image(&self, data: serde_json::Value) {
        let state = self.state.lock().await;
        let env = WsEnvelope {
            event_type: "image".to_string(),
            data,
        };
        let json = serde_json::to_string(&env).unwrap();
        for (_, client_tx) in state.clients.iter() {
            let _ = client_tx.send(json.clone());
        }
    }

    pub async fn broadcast_notice(&self, msg_text: String) {
        let mut state = self.state.lock().await;
        let notice_obj = NoticeMsg {
            msg: msg_text.clone(),
            time: Local::now().timestamp_millis(),
        };

        if msg_text.starts_with("Botty: Status Report") {
            state.botty_history.status_report = Some(notice_obj.clone());
        } else if msg_text.starts_with("Botty: Found") || msg_text.contains("Got stuck") {
            state.botty_history.found_records.push(notice_obj.clone());
            if state.botty_history.found_records.len() > 100 {
                state.botty_history.found_records.remove(0);
            }
        }

        let env = WsEnvelope {
            event_type: "notice".to_string(),
            data: serde_json::to_value(&notice_obj).unwrap(),
        };
        let json = serde_json::to_string(&env).unwrap();
        for (_, client_tx) in state.clients.iter() {
            let _ = client_tx.send(json.clone());
        }
    }

    pub async fn clear_history(&self) {
        let mut state = self.state.lock().await;
        state.msg_history.clear();
        state.botty_history.status_report = None;
        state.botty_history.found_records.clear();

        let env = WsEnvelope {
            event_type: "clear_history_client".to_string(),
            data: serde_json::Value::Null,
        };
        let json = serde_json::to_string(&env).unwrap();
        for (_, client_tx) in state.clients.iter() {
            let _ = client_tx.send(json.clone());
        }
    }
}
