//! codaKernel/1.1 audio, video and resource references in the embedded vocabulary.
//! The kernel only carries workspace-relative paths; reading the bytes belongs to the host.
use super::{Block, Content, Message, ToolCall};
pub use codacli_core::messages::{ContentBlock as CoreBlock, ImageSource as CoreImageSource};
use codacli_core::messages::{ToolCallBlock, format_media_block_placeholder};

/// Same rules as codaKernel/1.1: non-empty, workspace-relative, no control characters or `..`.
pub use codacli_core::messages::validate_kernel_media_path as validate_media_path;

/// Map host content to the equivalent `codacli_core` block.
pub fn core_content(content: &Content) -> CoreBlock {
    match content.clone() {
        Content::Text { text } => CoreBlock::Text {
            text,
            signature: None,
        },
        Content::Image { mime_type, data } => CoreBlock::Image {
            source: CoreImageSource::Base64 { mime_type, data },
        },
        Content::Audio {
            path,
            mime_type,
            bytes,
            duration_ms,
            sha256,
            name,
        } => CoreBlock::Audio {
            path,
            mime_type,
            bytes,
            duration_ms,
            sha256,
            name,
        },
        Content::Video {
            path,
            mime_type,
            bytes,
            duration_ms,
            sha256,
            name,
        } => CoreBlock::Video {
            path,
            mime_type,
            bytes,
            duration_ms,
            sha256,
            name,
        },
        Content::Resource {
            path,
            mime_type,
            bytes,
            sha256,
            name,
        } => CoreBlock::Resource {
            path,
            mime_type,
            bytes,
            sha256,
            name,
        },
    }
}

/// Map an assistant block to the equivalent `codacli_core` block.
pub fn core_block(block: &Block) -> CoreBlock {
    match block.clone() {
        Block::Text { text } => core_content(&Content::Text { text }),
        Block::Image { mime_type, data } => core_content(&Content::Image { mime_type, data }),
        Block::Thinking { text } => CoreBlock::Thinking {
            text,
            signature: None,
            encrypted_content: None,
        },
        Block::ToolCall {
            call: ToolCall { id, name, args },
        } => CoreBlock::ToolCall(ToolCallBlock {
            id,
            name,
            arguments: serde_json::Value::Object(args).to_string(),
            thought_signature: None,
            namespace: None,
        }),
        Block::Audio {
            path,
            mime_type,
            bytes,
            duration_ms,
            sha256,
            name,
        } => core_content(&Content::Audio {
            path,
            mime_type,
            bytes,
            duration_ms,
            sha256,
            name,
        }),
        Block::Video {
            path,
            mime_type,
            bytes,
            duration_ms,
            sha256,
            name,
        } => core_content(&Content::Video {
            path,
            mime_type,
            bytes,
            duration_ms,
            sha256,
            name,
        }),
        Block::Resource {
            path,
            mime_type,
            bytes,
            sha256,
            name,
        } => core_content(&Content::Resource {
            path,
            mime_type,
            bytes,
            sha256,
            name,
        }),
    }
}

/// Text projection for upstreams without native media input, e.g.
/// `[audio: clips/a.wav (audio/wav, 1500ms, intro)]`. `None` for text, images and tool calls.
pub fn format_media_placeholder(content: &Content) -> Option<String> {
    format_media_block_placeholder(&core_content(content))
}

/// [`format_media_placeholder`] for assistant blocks.
pub fn format_block_media_placeholder(block: &Block) -> Option<String> {
    format_media_block_placeholder(&core_block(block))
}

/// Truncate tool output to `max_chars` text characters like codaKernel/1.1: text parts are
/// joined, the head half and tail quarter are kept around an omission marker, and image and
/// media parts follow unchanged in their original order.
pub fn truncate_tool_content(content: Vec<Content>, max_chars: usize) -> Vec<Content> {
    let texts: Vec<&str> = content
        .iter()
        .filter_map(|part| match part {
            Content::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    if texts.iter().map(|t| t.chars().count()).sum::<usize>() <= max_chars {
        return content;
    }
    let (head_budget, tail_budget) = if max_chars >= 4 {
        (max_chars / 2, max_chars / 4)
    } else {
        (max_chars, 0)
    };
    let combined = texts.join("\n");
    let total = combined.chars().count();
    let head_end = combined
        .char_indices()
        .nth(head_budget)
        .map_or(combined.len(), |(i, _)| i);
    let tail_start = combined
        .char_indices()
        .nth(total.saturating_sub(tail_budget))
        .map_or(head_end, |(i, _)| i)
        .max(head_end);
    let omitted = total.saturating_sub(head_budget + tail_budget);
    let mut out = vec![Content::Text {
        text: format!(
            "{}\n[... truncated {omitted} chars ...]\n{}",
            &combined[..head_end],
            &combined[tail_start..]
        ),
    }];
    out.extend(
        content
            .into_iter()
            .filter(|part| !matches!(part, Content::Text { .. })),
    );
    out
}

/// codaKernel/1.1 summaries see media as fixed path markers instead of the files.
fn summary_marker(path: &str, mime_type: Option<&str>, kind: &str) -> String {
    match mime_type {
        Some(mime) => format!("[{kind}:{path} ({mime})]"),
        None => format!("[{kind}:{path}]"),
    }
}

fn summary_content(content: Content) -> Content {
    let text = match &content {
        Content::Audio {
            path, mime_type, ..
        } => summary_marker(path, Some(mime_type), "audio"),
        Content::Video {
            path, mime_type, ..
        } => summary_marker(path, Some(mime_type), "video"),
        Content::Resource { path, .. } => summary_marker(path, None, "resource"),
        Content::Text { .. } | Content::Image { .. } => return content,
    };
    Content::Text { text }
}

fn summary_block(block: Block) -> Block {
    let text = match &block {
        Block::Audio {
            path, mime_type, ..
        } => summary_marker(path, Some(mime_type), "audio"),
        Block::Video {
            path, mime_type, ..
        } => summary_marker(path, Some(mime_type), "video"),
        Block::Resource { path, .. } => summary_marker(path, None, "resource"),
        Block::Text { .. }
        | Block::Image { .. }
        | Block::Thinking { .. }
        | Block::ToolCall { .. } => {
            return block;
        }
    };
    Block::Text { text }
}

/// Replace media references in a compaction request with the codaKernel/1.1 summary markers.
pub(super) fn summary_projection(message: Message) -> Message {
    let map = |content: Vec<Content>| content.into_iter().map(summary_content).collect();
    match message {
        Message::System { content } => Message::System {
            content: map(content),
        },
        Message::User { content } => Message::User {
            content: map(content),
        },
        Message::Custom(mut custom) => {
            custom.content = map(custom.content);
            Message::Custom(custom)
        }
        Message::Assistant { blocks } => Message::Assistant {
            blocks: blocks.into_iter().map(summary_block).collect(),
        },
        Message::Tool {
            call_id,
            name,
            mut result,
        } => {
            result.content = map(result.content);
            Message::Tool {
                call_id,
                name,
                result,
            }
        }
        other @ (Message::Aborted { .. } | Message::Summary { .. }) => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn audio() -> Content {
        Content::Audio {
            path: "clips/a.wav".into(),
            mime_type: "audio/wav".into(),
            bytes: Some(10),
            duration_ms: Some(1500),
            sha256: None,
            name: Some("intro".into()),
        }
    }

    #[test]
    fn placeholders_match_the_core_projection() {
        assert_eq!(
            format_media_placeholder(&audio()).as_deref(),
            Some("[audio: clips/a.wav (audio/wav, 1500ms, intro)]")
        );
        let resource = Block::Resource {
            path: "docs/a.pdf".into(),
            mime_type: None,
            bytes: None,
            sha256: None,
            name: None,
        };
        assert_eq!(
            format_block_media_placeholder(&resource).as_deref(),
            Some("[resource: docs/a.pdf]")
        );
        assert_eq!(
            format_media_placeholder(&Content::Text { text: "x".into() }),
            None
        );
    }

    #[test]
    fn core_mapping_keeps_reference_fields_without_inlining() {
        let CoreBlock::Audio {
            path,
            bytes,
            duration_ms,
            name,
            ..
        } = core_content(&audio())
        else {
            panic!("expected audio")
        };
        assert_eq!(
            (path.as_str(), bytes, duration_ms, name.as_deref()),
            ("clips/a.wav", Some(10), Some(1500), Some("intro"))
        );
        assert!(validate_media_path("clips/a.wav").is_ok());
        for bad in ["", "/abs.wav", "C:/a.wav", "a/../b.wav", "a\nb"] {
            assert!(validate_media_path(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn truncation_counts_only_text_and_keeps_media_in_order() {
        let image = Content::Image {
            mime_type: "image/png".into(),
            data: "AAAA".into(),
        };
        let content = vec![
            Content::Text {
                text: "a".repeat(30),
            },
            audio(),
            image.clone(),
            Content::Text {
                text: "b".repeat(30),
            },
        ];
        assert_eq!(truncate_tool_content(content.clone(), 61), content);
        let out = truncate_tool_content(content, 20);
        let [Content::Text { text }, media, img] = out.as_slice() else {
            panic!("{out:?}")
        };
        assert_eq!(
            text,
            &format!(
                "{}\n[... truncated 46 chars ...]\n{}",
                "a".repeat(10),
                "b".repeat(5)
            )
        );
        assert_eq!((media, img), (&audio(), &image));
    }

    #[test]
    fn summary_projection_uses_codakernel_markers() {
        let message = Message::Tool {
            call_id: "c".into(),
            name: "render".into(),
            result: super::super::ToolResult {
                content: vec![audio()],
                details: serde_json::Value::Null,
                is_error: false,
            },
        };
        let Message::Tool { result, .. } = summary_projection(message) else {
            panic!()
        };
        assert_eq!(
            result.content,
            vec![Content::Text {
                text: "[audio:clips/a.wav (audio/wav)]".into()
            }]
        );
    }
}
