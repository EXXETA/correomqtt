use std::path::{Path, PathBuf};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{Result, StorageError};

use super::{Message, PublishMessageHistory, PublishTopicHistory, SubscriptionHistory};

pub const MAX_HISTORY_ENTRIES: usize = 100;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConnectionHistorySnapshot {
    pub connection_id: String,
    pub publish_topics: PublishTopicHistory,
    pub publish_messages: PublishMessageHistory,
    pub subscriptions: SubscriptionHistory,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HistoryPersistenceSnapshot {
    pub connections: std::collections::BTreeMap<String, ConnectionHistorySnapshot>,
}

#[derive(Clone, Debug)]
pub struct HistoryStore {
    root: PathBuf,
}

impl HistoryStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn load_connection(&self, connection_id: &str) -> Result<ConnectionHistorySnapshot> {
        Ok(ConnectionHistorySnapshot {
            connection_id: connection_id.to_owned(),
            publish_topics: self.load_publish_topics(connection_id)?,
            publish_messages: self.load_publish_messages(connection_id)?,
            subscriptions: self.load_subscriptions(connection_id)?,
        })
    }

    pub fn load_workbench<T>(&self, connection_id: &str) -> Result<T>
    where
        T: Default + DeserializeOwned,
    {
        self.read_or_default(self.workbench_path(connection_id))
    }

    pub fn replace_workbench<T>(&self, connection_id: &str, workbench: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.write_json(self.workbench_path(connection_id), workbench)
    }

    pub fn replace_all(&self, snapshot: &HistoryPersistenceSnapshot) -> Result<()> {
        for connection in snapshot.connections.values() {
            self.replace_connection_history(connection)?;
        }
        Ok(())
    }

    pub fn replace_connection_history(&self, snapshot: &ConnectionHistorySnapshot) -> Result<()> {
        self.write_json(
            self.path(&snapshot.connection_id, HistoryFileKind::PublishTopics),
            &snapshot.publish_topics,
        )?;
        self.write_json(
            self.path(&snapshot.connection_id, HistoryFileKind::PublishMessages),
            &snapshot.publish_messages,
        )?;
        self.write_json(
            self.path(&snapshot.connection_id, HistoryFileKind::Subscriptions),
            &snapshot.subscriptions,
        )
    }

    pub fn load_publish_topics(&self, connection_id: &str) -> Result<PublishTopicHistory> {
        self.read_or_default(self.path(connection_id, HistoryFileKind::PublishTopics))
    }

    pub fn load_publish_messages(&self, connection_id: &str) -> Result<PublishMessageHistory> {
        self.read_or_default(self.path(connection_id, HistoryFileKind::PublishMessages))
    }

    pub fn load_subscriptions(&self, connection_id: &str) -> Result<SubscriptionHistory> {
        self.read_or_default(self.path(connection_id, HistoryFileKind::Subscriptions))
    }

    pub fn record_publish_success(
        &self,
        connection_id: &str,
        message: Message,
    ) -> Result<ConnectionHistorySnapshot> {
        let topic = message.topic.clone();
        self.record_publish_topic(connection_id, topic)?;
        self.record_publish_message(connection_id, message)?;
        self.load_connection(connection_id)
    }

    pub fn record_publish_topic(
        &self,
        connection_id: &str,
        topic: impl Into<String>,
    ) -> Result<PublishTopicHistory> {
        let mut history = self.load_publish_topics(connection_id)?;
        push_recent_unique(&mut history.topics, topic.into());
        let path = self.path(connection_id, HistoryFileKind::PublishTopics);
        self.write_json(path, &history)?;
        Ok(history)
    }

    pub fn record_publish_message(
        &self,
        connection_id: &str,
        message: Message,
    ) -> Result<PublishMessageHistory> {
        let mut history = self.load_publish_messages(connection_id)?;
        history.messages.insert(0, message);
        history.messages.truncate(MAX_HISTORY_ENTRIES);
        let path = self.path(connection_id, HistoryFileKind::PublishMessages);
        self.write_json(path, &history)?;
        Ok(history)
    }

    pub fn remove_published_message(
        &self,
        connection_id: &str,
        message: &Message,
    ) -> Result<PublishMessageHistory> {
        let mut history = self.load_publish_messages(connection_id)?;
        if let Some(index) = history.messages.iter().position(|entry| entry == message) {
            history.messages.remove(index);
        }
        let path = self.path(connection_id, HistoryFileKind::PublishMessages);
        self.write_json(path, &history)?;
        Ok(history)
    }

    pub fn clear_published_messages(&self, connection_id: &str) -> Result<PublishMessageHistory> {
        let history = PublishMessageHistory::default();
        let path = self.path(connection_id, HistoryFileKind::PublishMessages);
        self.write_json(path, &history)?;
        Ok(history)
    }

    pub fn record_subscription(
        &self,
        connection_id: &str,
        topic: impl Into<String>,
        hidden: bool,
    ) -> Result<SubscriptionHistory> {
        let mut history = self.load_subscriptions(connection_id)?;
        if hidden {
            return Ok(history);
        }
        push_recent_unique(&mut history.topics, topic.into());
        let path = self.path(connection_id, HistoryFileKind::Subscriptions);
        self.write_json(path, &history)?;
        Ok(history)
    }

    fn path(&self, connection_id: &str, kind: HistoryFileKind) -> PathBuf {
        self.root
            .join(format!("{connection_id}_{}", kind.file_name()))
    }

    fn workbench_path(&self, connection_id: &str) -> PathBuf {
        self.root.join(format!("{connection_id}.json"))
    }

    fn read_or_default<T>(&self, path: PathBuf) -> Result<T>
    where
        T: Default + DeserializeOwned,
    {
        if !path.exists() {
            return Ok(T::default());
        }
        let text = std::fs::read_to_string(&path).map_err(|source| StorageError::Read {
            path: path.clone(),
            source,
        })?;
        serde_json::from_str(&text).map_err(|source| StorageError::Json { path, source })
    }

    fn write_json<T>(&self, path: PathBuf, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        std::fs::create_dir_all(&self.root).map_err(|source| StorageError::CreateDir {
            path: self.root.clone(),
            source,
        })?;
        let text = serde_json::to_string_pretty(value).map_err(|source| StorageError::Json {
            path: path.clone(),
            source,
        })?;
        write_file_atomic(&path, text.as_bytes())
    }
}

fn write_file_atomic(path: &Path, content: &[u8]) -> Result<()> {
    let temporary = path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&temporary, content).map_err(|source| StorageError::Write {
        path: temporary.clone(),
        source,
    })?;
    if cfg!(windows) && path.exists() {
        std::fs::remove_file(path).map_err(|source| StorageError::Write {
            path: path.to_path_buf(),
            source,
        })?;
    }
    std::fs::rename(&temporary, path).map_err(|source| StorageError::Write {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum HistoryFileKind {
    PublishTopics,
    PublishMessages,
    Subscriptions,
}

impl HistoryFileKind {
    fn file_name(self) -> &'static str {
        match self {
            Self::PublishTopics => "publishHistory.json",
            Self::PublishMessages => "publishMessageHistory.json",
            Self::Subscriptions => "subscriptionHistory.json",
        }
    }
}

fn push_recent_unique(entries: &mut Vec<String>, topic: String) {
    entries.retain(|entry| entry != &topic);
    entries.push(topic);
    while entries.len() > MAX_HISTORY_ENTRIES {
        entries.remove(0);
    }
}
