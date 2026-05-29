use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

const RANDOM_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const RANDOM_SUFFIX_LEN: usize = 14;
const TIME_HEX_LEN: usize = 12;
#[cfg(test)]
const ID_BODY_LEN: usize = TIME_HEX_LEN + RANDOM_SUFFIX_LEN;
const TIME_MASK: u64 = 0x0000_ffff_ffff_ffff;
static LAST_ID_SEQUENCE: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0));

pub fn session_id() -> String {
    prefixed("ses")
}

pub fn message_id() -> String {
    prefixed("msg")
}

pub fn message_id_after(parent_id: &str) -> String {
    prefixed_after("msg", parent_id)
}

pub fn part_id() -> String {
    prefixed("prt")
}

pub fn project_id() -> String {
    prefixed("prj")
}

pub fn workspace_id() -> String {
    prefixed("wrk")
}

pub fn request_id() -> String {
    prefixed("req")
}

pub fn slug(input: &str) -> String {
    let mut result = String::new();
    let mut previous_dash = false;
    for ch in input.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            result.push(ch);
            previous_dash = false;
        } else if !previous_dash {
            result.push('-');
            previous_dash = true;
        }
    }
    let trimmed = result.trim_matches('-');
    if trimmed.is_empty() {
        "session".to_string()
    } else {
        trimmed.to_string()
    }
}

fn prefixed(prefix: &str) -> String {
    format_id(prefix, next_sequence())
}

fn prefixed_after(prefix: &str, parent_id: &str) -> String {
    let sequence = parent_id
        .strip_prefix(&format!("{prefix}_"))
        .and_then(|body| body.get(..TIME_HEX_LEN))
        .and_then(|time| u64::from_str_radix(time, 16).ok())
        .map_or_else(next_sequence, |parent_sequence| {
            next_sequence().max(parent_sequence.saturating_add(1))
        });
    format_id(prefix, sequence)
}

fn next_sequence() -> u64 {
    let base = unix_ms() << 12;
    let mut last = LAST_ID_SEQUENCE.lock().expect("id state lock");
    let sequence = base.max(last.saturating_add(1));
    *last = sequence;
    sequence & TIME_MASK
}

fn format_id(prefix: &str, sequence: u64) -> String {
    format!(
        "{prefix}_{:0width$x}{}",
        sequence & TIME_MASK,
        random_base62(),
        width = TIME_HEX_LEN
    )
}

fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_millis() as u64
}

fn random_base62() -> String {
    let bytes = *Uuid::new_v4().as_bytes();
    let mut result = String::with_capacity(RANDOM_SUFFIX_LEN);
    for byte in bytes.iter().take(RANDOM_SUFFIX_LEN) {
        result.push(RANDOM_CHARS[*byte as usize % RANDOM_CHARS.len()] as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_ids_sort_in_creation_order() {
        let ids = (0..100).map(|_| message_id()).collect::<Vec<_>>();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn message_ids_match_opencode_body_width() {
        let id = message_id();
        let body = id.strip_prefix("msg_").expect("message id prefix");
        assert_eq!(body.len(), ID_BODY_LEN);
    }

    #[test]
    fn assistant_message_ids_sort_after_client_parent_ids() {
        let parent_time = message_id()
            .strip_prefix("msg_")
            .expect("message id prefix")
            .get(..TIME_HEX_LEN)
            .expect("time prefix")
            .to_string();
        let parent_id = format!("msg_{parent_time}zzzzzzzzzzzzzz");
        let assistant_id = message_id_after(&parent_id);

        assert!(
            assistant_id > parent_id,
            "assistant id {assistant_id} must sort after parent id {parent_id}"
        );
    }

    #[test]
    fn part_ids_sort_in_creation_order() {
        let ids = (0..100).map(|_| part_id()).collect::<Vec<_>>();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }
}
