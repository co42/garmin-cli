//! Tracing setup for `-v` / `--verbose`. Emits one compact line per Garmin
//! API request and response to stderr.

use colored::Colorize;
use tracing_subscriber::EnvFilter;

pub fn init() {
    // RUST_LOG wins if set; otherwise show our API events at DEBUG.
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("garmin::api=debug"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .event_format(ApiFormatter)
        .init();
}

struct ApiFormatter;

impl<S, N> tracing_subscriber::fmt::FormatEvent<S, N> for ApiFormatter
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> tracing_subscriber::fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let mut fields = ApiFields::default();
        event.record(&mut fields);

        match fields.message.as_deref() {
            Some("request") => {
                let method = fields.method.as_deref().unwrap_or("?");
                let url = fields.url.as_deref().unwrap_or("?");
                writeln!(writer, "{} {} {}", "→".cyan(), method.bold(), url.dimmed())?;
                if let Some(body) = fields.body.as_deref().filter(|b| !b.is_empty()) {
                    writeln!(writer, "  {} {}", "body:".dimmed(), body)?;
                }
            }
            Some(msg) if msg.starts_with("response") => {
                let status = fields.status.as_deref().unwrap_or("?");
                let elapsed = fields.elapsed_ms.as_deref().unwrap_or("?");
                let bytes = fields.body_bytes.as_deref().unwrap_or("?");
                let status_colored = if status.starts_with('2') {
                    status.green()
                } else if status.starts_with('4') || status.starts_with('5') {
                    status.red()
                } else {
                    status.yellow()
                };
                writeln!(
                    writer,
                    "{} {} {} {}",
                    "←".cyan(),
                    status_colored,
                    format!("({elapsed}ms, {bytes}B)").dimmed(),
                    if msg == "response (binary)" {
                        "[binary]".dimmed().to_string()
                    } else {
                        String::new()
                    }
                )?;
                if let Some(body) = fields.body.as_deref().filter(|b| !b.is_empty()) {
                    writeln!(writer, "  {} {}", "body:".dimmed(), body)?;
                }
            }
            _ => {
                // Fallback: render a plain message with fields.
                if let Some(msg) = fields.message.as_deref() {
                    write!(writer, "{msg}")?;
                }
                writeln!(writer)?;
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct ApiFields {
    message: Option<String>,
    method: Option<String>,
    url: Option<String>,
    status: Option<String>,
    elapsed_ms: Option<String>,
    body_bytes: Option<String>,
    body: Option<String>,
}

impl ApiFields {
    fn slot(&mut self, name: &str) -> Option<&mut Option<String>> {
        match name {
            "message" => Some(&mut self.message),
            "method" => Some(&mut self.method),
            "url" => Some(&mut self.url),
            "status" => Some(&mut self.status),
            "elapsed_ms" => Some(&mut self.elapsed_ms),
            "body_bytes" => Some(&mut self.body_bytes),
            "body" => Some(&mut self.body),
            _ => None,
        }
    }
}

impl tracing::field::Visit for ApiFields {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if let Some(slot) = self.slot(field.name()) {
            *slot = Some(format!("{value:?}").trim_matches('"').to_string());
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if let Some(slot) = self.slot(field.name()) {
            *slot = Some(value.to_string());
        }
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        if let Some(slot) = self.slot(field.name()) {
            *slot = Some(value.to_string());
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if let Some(slot) = self.slot(field.name()) {
            *slot = Some(value.to_string());
        }
    }
}
