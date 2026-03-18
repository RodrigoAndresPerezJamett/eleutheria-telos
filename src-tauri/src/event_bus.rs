// Phase 1: ClipboardChanged, NoteCreated, NoteUpdated are now wired to handlers.
// Remaining events (OcrCompleted, TranscriptionCompleted, etc.) will be wired in Phase 2+.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

pub const CHANNEL_CAPACITY: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Event {
    OcrCompleted {
        text: String,
        source: String,
    },
    TranscriptionCompleted {
        text: String,
        language: String,
    },
    ClipboardChanged {
        content: String,
        content_type: String,
    },
    NoteCreated {
        id: String,
        title: String,
    },
    NoteUpdated {
        id: String,
    },
    TranslationCompleted {
        original: String,
        translated: String,
        target_lang: String,
    },
    RecordingStarted {
        recording_type: String,
    },
    RecordingStopped {
        file_path: String,
    },
}

#[derive(Clone)]
pub struct EventBus {
    sender: Arc<broadcast::Sender<Event>>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(CHANNEL_CAPACITY);
        Self {
            sender: Arc::new(sender),
        }
    }

    pub fn publish(&self, event: Event) {
        // Ignore error when there are no subscribers
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
