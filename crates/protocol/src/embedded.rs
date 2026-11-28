//! In-process kernel vocabulary. Storage revisions, model identifiers and tool schemas
//! belong to the embedding host. These types are independent of the SQLite event schema.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub provider: Option<String>,
}

/// Media blocks carry workspace-relative paths, never inline base64. Field names follow
/// `codacli_core::messages::ContentBlock`. Hosts that only send text upstream can project them
/// with `coda::embedded::format_media_placeholder`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Content {
    Text {
        text: String,
    },
    Image {
        mime_type: String,
        data: String,
    },
    /// codaKernel/1.1 reference to a workspace-relative audio file. Bytes stay with the host.
    Audio {
        path: String,
        mime_type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bytes: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sha256: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// codaKernel/1.1 reference to a workspace-relative video file.
    Video {
        path: String,
        mime_type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bytes: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sha256: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// codaKernel/1.1 reference to any other workspace-relative file.
    Resource {
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bytes: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sha256: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
}

/// Archival form of a host message. Only `content` is projected into the model request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CustomMessage {
    pub custom_type: String,
    pub content: Vec<Content>,
    pub display: bool,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_value"
    )]
    pub details: Option<Value>,
}

// A present JSON null is meaningful in the host's session file.
fn present_value<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<Value>, D::Error> {
    Value::deserialize(d).map(Some)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SteeringMessage {
    User { content: Vec<Content> },
    Custom(CustomMessage),
}

impl From<SteeringMessage> for Message {
    fn from(input: SteeringMessage) -> Self {
        match input {
            SteeringMessage::User { content } => Self::User { content },
            SteeringMessage::Custom(custom) => Self::Custom(custom),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: Map<String, Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Vec<Content>,
    pub details: Value,
    pub is_error: bool,
}

/// Assistant blocks. Media variants match the corresponding [`Content`] variants.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Block {
    Text {
        text: String,
    },
    Image {
        mime_type: String,
        data: String,
    },
    /// See [`Content::Audio`].
    Audio {
        path: String,
        mime_type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bytes: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sha256: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// See [`Content::Video`].
    Video {
        path: String,
        mime_type: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bytes: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sha256: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// See [`Content::Resource`].
    Resource {
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bytes: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sha256: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    Thinking {
        text: String,
    },
    ToolCall {
        call: ToolCall,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Message {
    System {
        content: Vec<Content>,
    },
    User {
        content: Vec<Content>,
    },
    Custom(CustomMessage),
    Assistant {
        blocks: Vec<Block>,
    },
    /// An assistant message closed by the user. Partial blocks are archival only.
    Aborted {
        run_id: String,
        partial: Vec<Block>,
    },
    Tool {
        call_id: String,
        name: String,
        result: ToolResult,
    },
    Summary {
        text: String,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Entry {
    Message(Message),
    Usage(Usage),
    /// Summarizes the complete selected projection at the expected revision.
    Compaction {
        summary: String,
        /// Estimate supplied by the host at a generation boundary.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tokens_before: Option<u64>,
    },
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub revision: String,
    pub messages: Vec<Message>,
}

/// Per-request limits are checked against the selected provider by `preflight`.
#[derive(Clone, Debug, Default)]
pub struct ModelSettings {
    pub thinking_level: Option<String>,
    pub context_window: Option<usize>,
    pub max_output_tokens: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct RunOptions {
    pub session_id: String,
    pub run_id: String,
    pub model: Model,
    pub system_prompt: String,
    pub content: Vec<Content>,
    pub max_requests: usize,
    pub settings: ModelSettings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestPurpose {
    Generation,
    Compaction,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CompactionDecision {
    #[default]
    Continue,
    Compact {
        instructions: String,
        tokens_before: u64,
    },
}

#[derive(Clone, Debug)]
pub struct ModelRequest {
    pub purpose: RequestPurpose,
    pub session_id: String,
    /// Present throughout a run, including its summaries. Explicit compaction has no run ID.
    pub run_id: Option<String>,
    pub settings: ModelSettings,
    pub model: Model,
    pub system_prompt: String,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolDefinition>,
}

#[derive(Clone, Debug)]
pub enum ModelDelta {
    Text { block_id: String, delta: String },
    Thinking { block_id: String, delta: String },
}

#[derive(Clone, Debug)]
pub struct ModelReply {
    /// Complete, successfully finished blocks; a truncated/failed generation is an error.
    pub blocks: Vec<Block>,
    pub usage: Option<Usage>,
}

#[derive(Debug)]
pub struct Invocation {
    pub session_id: String,
    pub run_id: String,
    pub call: ToolCall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoneReason {
    Completed,
    Aborted,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    Start {
        model: Model,
    },
    CompactionStart {
        tokens_before: u64,
    },
    CompactionEnd {
        reason: DoneReason,
    },
    /// Emitted once per queued input, after its batch was committed to the session.
    InputDelivered {
        message: SteeringMessage,
    },
    Text {
        block_id: String,
        delta: String,
    },
    Thinking {
        block_id: String,
        delta: String,
    },
    ToolCall {
        call: ToolCall,
    },
    ToolUpdate {
        call_id: String,
        result: ToolResult,
    },
    ToolResult {
        call_id: String,
        result: ToolResult,
    },
    Usage {
        usage: Usage,
    },
    Error {
        code: String,
        message: String,
    },
    Done {
        reason: DoneReason,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_references_omit_absent_metadata_and_round_trip() {
        let audio = Content::Audio {
            path: "media/clip.wav".into(),
            mime_type: "audio/wav".into(),
            bytes: None,
            duration_ms: Some(1500),
            sha256: None,
            name: None,
        };
        let json = serde_json::to_value(&audio).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"Audio": {"path": "media/clip.wav", "mime_type": "audio/wav", "duration_ms": 1500}})
        );
        assert_eq!(serde_json::from_value::<Content>(json).unwrap(), audio);
        let resource: Block =
            serde_json::from_value(serde_json::json!({"Resource": {"path": "docs/a.pdf"}}))
                .unwrap();
        assert_eq!(
            resource,
            Block::Resource {
                path: "docs/a.pdf".into(),
                mime_type: None,
                bytes: None,
                sha256: None,
                name: None,
            }
        );
    }

    #[test]
    fn legacy_compaction_entries_keep_their_wire_shape() {
        let legacy = serde_json::json!({"Compaction": {"summary": "earlier"}});
        let entry: Entry = serde_json::from_value(legacy.clone()).unwrap();
        assert!(matches!(
            entry,
            Entry::Compaction {
                tokens_before: None,
                ..
            }
        ));
        assert_eq!(serde_json::to_value(entry).unwrap(), legacy);
    }

    #[test]
    fn custom_details_round_trip_missing_null_and_structured_json() {
        for details in [
            None,
            Some(Value::Null),
            Some(serde_json::json!({"count": 2})),
        ] {
            let message = CustomMessage {
                custom_type: "report".into(),
                content: vec![Content::Text {
                    text: "done".into(),
                }],
                display: false,
                details,
            };
            let json = serde_json::to_value(&message).unwrap();
            assert_eq!(json.get("details"), message.details.as_ref());
            assert_eq!(
                serde_json::from_value::<CustomMessage>(json).unwrap(),
                message
            );
        }
    }
}
