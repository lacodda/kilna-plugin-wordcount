//! A kilna plugin: counts what a draft actually contains.
//!
//! Two calls, following the line's plugin convention:
//!
//!   kilna-plugin-wordcount --manifest   → what this plugin offers
//!   kilna-plugin-wordcount run          → JSON invocation on stdin, outcome on stdout
//!
//! It lives for the duration of a call and holds no state.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

const PROTOCOL_VERSION: u32 = 1;

/// Average adult reading speed, words per minute. Rounded down from the usual
/// 200–250 range: a draft is read more slowly than finished prose.
const WORDS_PER_MINUTE: f64 = 200.0;

#[derive(Debug, Deserialize)]
struct Invocation {
    #[allow(dead_code)]
    command: String,
    /// The work or release the command was invoked on.
    subject: Value,
}

#[derive(Debug, Default, Serialize)]
struct Outcome {
    #[serde(skip_serializing_if = "Map::is_empty")]
    meta: Map<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn main() {
    let argument = std::env::args().nth(1).unwrap_or_default();

    match argument.as_str() {
        "--manifest" => print!("{}", manifest()),
        "run" => run(),
        other => {
            eprintln!("unknown argument `{other}`; expected --manifest or run");
            std::process::exit(2);
        }
    }
}

fn manifest() -> String {
    json!({
        "protocol_version": PROTOCOL_VERSION,
        "name": "wordcount",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Counts words, lines and reading time for the current draft.",
        "commands": [{
            "key": "count",
            "label": "Count words",
            "description": "Words, lines and an estimated reading time, written into the work's fields.",
            "target": "work"
        }]
    })
    .to_string()
}

fn run() {
    let mut input = String::new();
    if let Err(error) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut input) {
        fail(&format!("could not read the request: {error}"));
    }

    let invocation: Invocation = match serde_json::from_str(&input) {
        Ok(invocation) => invocation,
        Err(error) => fail(&format!("could not read the request: {error}")),
    };

    // kilna sends every role's latest body under `bodies`.
    let bodies = invocation
        .subject
        .get("bodies")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    // The longest body is the one worth measuring: a style prompt is not the
    // work, and picking by role name would tie this to one craft.
    let Some((role, text)) = bodies
        .iter()
        .filter_map(|(role, value)| value.as_str().map(|text| (role.clone(), text)))
        .max_by_key(|(_, text)| text.chars().count())
    else {
        emit(Outcome {
            message: Some("Nothing written yet.".into()),
            ..Default::default()
        });
        return;
    };

    let words = count_words(text);
    let lines = text.lines().filter(|line| !line.trim().is_empty()).count();
    let minutes = (words as f64 / WORDS_PER_MINUTE).ceil().max(1.0) as u64;

    let mut meta = Map::new();
    meta.insert("words".into(), json!(words));
    meta.insert("lines".into(), json!(lines));
    meta.insert("reading_minutes".into(), json!(minutes));

    emit(Outcome {
        meta,
        message: Some(format!(
            "{words} words over {lines} lines in the {role} — about {minutes} min to read."
        )),
        error: None,
    });
}

/// Words, counted the way a writer would: whitespace-separated runs that
/// contain at least one letter or digit, so a lone dash is not a word.
fn count_words(text: &str) -> usize {
    text.split_whitespace()
        .filter(|token| token.chars().any(char::is_alphanumeric))
        .count()
}

fn emit(outcome: Outcome) {
    match serde_json::to_string(&outcome) {
        Ok(json) => print!("{json}"),
        Err(error) => fail(&format!("could not write the reply: {error}")),
    }
}

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_manifest_is_valid_json_and_declares_the_protocol() {
        let parsed: Value = serde_json::from_str(&manifest()).unwrap();

        assert_eq!(parsed["protocol_version"], json!(PROTOCOL_VERSION));
        assert_eq!(parsed["commands"][0]["target"], json!("work"));
    }

    #[test]
    fn words_are_whitespace_runs_containing_something_readable() {
        assert_eq!(count_words("the cranes go still"), 4);
        assert_eq!(count_words("a — b"), 2, "a lone dash is not a word");
        assert_eq!(count_words("  spaced   out  "), 2);
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn cyrillic_counts_the_same_as_latin() {
        assert_eq!(count_words("тёплые соты горят"), 3);
    }

    #[test]
    fn reading_time_is_never_reported_as_zero() {
        let minutes = (3.0_f64 / WORDS_PER_MINUTE).ceil().max(1.0) as u64;

        assert_eq!(minutes, 1, "a three-word draft still takes a moment");
    }
}
