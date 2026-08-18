use crate::models::{NoticeMsg, SayMsg};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct ChatStorageData {
    pub say_history: Vec<SayMsg>,
    pub botty_status: Option<NoticeMsg>,
    pub botty_records: Vec<NoticeMsg>,
}

#[derive(Clone)]
pub struct Storage {
    file_path: PathBuf,
    data: Arc<Mutex<ChatStorageData>>,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(db_path: P) -> std::io::Result<Self> {
        let file_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = file_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut data = ChatStorageData::default();
        if file_path.exists() {
            if let Ok(mut file) = File::open(&file_path) {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() && !contents.trim().is_empty() {
                    if let Ok(loaded) = serde_json::from_str::<ChatStorageData>(&contents) {
                        data = loaded;
                    }
                }
            }
        }

        Ok(Storage {
            file_path,
            data: Arc::new(Mutex::new(data)),
        })
    }

    fn persist(&self) {
        if let Ok(guard) = self.data.lock() {
            if let Ok(json_str) = serde_json::to_string_pretty(&*guard) {
                let temp_path = self.file_path.with_extension("tmp");
                if let Ok(mut file) = File::create(&temp_path) {
                    if file.write_all(json_str.as_bytes()).is_ok() {
                        let _ = fs::rename(&temp_path, &self.file_path);
                    }
                }
            }
        }
    }

    pub fn save_say_msg(&self, msg: &SayMsg) {
        if let Ok(mut guard) = self.data.lock() {
            guard.say_history.push(msg.clone());
            if guard.say_history.len() > 100 {
                guard.say_history.remove(0);
            }
        }
        self.persist();
    }

    pub fn load_recent_say_msgs(&self, limit: usize) -> Vec<SayMsg> {
        if let Ok(guard) = self.data.lock() {
            let len = guard.say_history.len();
            let start = if len > limit { len - limit } else { 0 };
            guard.say_history[start..].to_vec()
        } else {
            Vec::new()
        }
    }

    pub fn save_botty_status(&self, status: &NoticeMsg) {
        if let Ok(mut guard) = self.data.lock() {
            guard.botty_status = Some(status.clone());
        }
        self.persist();
    }

    pub fn save_botty_found(&self, found: &NoticeMsg) {
        if let Ok(mut guard) = self.data.lock() {
            guard.botty_records.push(found.clone());
            if guard.botty_records.len() > 100 {
                guard.botty_records.remove(0);
            }
        }
        self.persist();
    }

    pub fn load_botty_history(&self) -> (Option<NoticeMsg>, Vec<NoticeMsg>) {
        if let Ok(guard) = self.data.lock() {
            (guard.botty_status.clone(), guard.botty_records.clone())
        } else {
            (None, Vec::new())
        }
    }

    pub fn clear_all_history(&self) {
        if let Ok(mut guard) = self.data.lock() {
            guard.say_history.clear();
            guard.botty_status = None;
            guard.botty_records.clear();
        }
        self.persist();
    }
}
