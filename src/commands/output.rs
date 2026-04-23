use colored::Colorize;
use serde::Serialize;

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Output {
    json: bool,
    fields: Vec<String>,
}

impl Output {
    pub fn new(json: bool, fields: Vec<String>) -> Self {
        Self { json, fields }
    }

    pub fn print<T: Serialize + HumanReadable>(&self, data: &T) {
        if self.json {
            self.print_json(data);
        } else {
            data.print_human();
        }
    }

    pub fn print_list<T: Serialize + HumanReadable>(&self, items: &[T], title: &str) {
        if self.json {
            self.print_json(&items);
            return;
        }
        println!("{}", title.bold());
        println!("{}", "\u{2500}".repeat(40).dimmed());
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                println!();
            }
            item.print_human();
        }
        println!();
        println!("{} item{}", items.len(), if items.len() == 1 { "" } else { "s" });
    }

    /// Like `print_list` but prints rows contiguously without blank separators.
    /// Use for tabular snapshots (zones, splits, laps) where each row is a
    /// single line and blank-separation would waste vertical space.
    pub fn print_table<T: Serialize + HumanReadable>(&self, items: &[T], title: &str) {
        if self.json {
            self.print_json(&items);
            return;
        }
        println!("{}", title.bold());
        println!("{}", "\u{2500}".repeat(40).dimmed());
        for item in items {
            item.print_human();
        }
    }

    /// Print a raw `serde_json::Value`, respecting `--fields`.
    pub fn print_value(&self, value: &serde_json::Value) {
        let filtered = self.filter_fields(value.clone());
        println!("{}", self.serialize_json(&filtered));
    }

    fn print_json<T: Serialize>(&self, data: &T) {
        let value = serde_json::to_value(data).unwrap();
        let filtered = self.filter_fields(value);
        println!("{}", self.serialize_json(&filtered));
    }

    fn serialize_json<T: Serialize>(&self, data: &T) -> String {
        serde_json::to_string_pretty(data).unwrap()
    }

    fn filter_fields(&self, value: serde_json::Value) -> serde_json::Value {
        if self.fields.is_empty() {
            return value;
        }
        match value {
            serde_json::Value::Object(map) => {
                let filtered: serde_json::Map<String, serde_json::Value> = map
                    .into_iter()
                    .filter(|(k, _)| self.fields.iter().any(|f| f == k))
                    .collect();
                serde_json::Value::Object(filtered)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(|v| self.filter_fields(v)).collect())
            }
            other => other,
        }
    }

    /// Print a structured error. JSON mode emits `{"error": "...", "code": "..."}` to stderr.
    pub fn error_structured(&self, err: &Error) {
        if self.json {
            let obj = serde_json::json!({
                "error": err.to_string(),
                "code": err.code(),
            });
            eprintln!("{}", serde_json::to_string_pretty(&obj).unwrap());
        } else {
            eprintln!("{} {}", "\u{2717}".red(), err);
        }
    }

    pub fn success(&self, msg: &str) {
        if !self.json {
            println!("{} {}", "\u{2713}".green(), msg);
        }
    }

    pub fn status(&self, msg: &str) {
        if !self.json {
            eprintln!("{}", msg.dimmed());
        }
    }

    pub fn is_json(&self) -> bool {
        self.json
    }
}

pub trait HumanReadable {
    fn print_human(&self);
}

/// Shared column width for "  Label: value" rows across human output.
/// Longest label is ~14 chars ("Fitness trend:"); pad to 16 for breathing room.
pub const LABEL_WIDTH: usize = 16;
