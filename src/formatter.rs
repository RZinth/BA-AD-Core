use crate::utils::{contains_url, format_urls, get_level_visual_length, is_cause_section, level_to_index};

use chrono::{DateTime, Local};
use owo_colors::OwoColorize;
use smallvec::SmallVec;
use std::fmt;
use std::sync::Arc;
use tracing::{
    field::{Field, Visit}, Event, Level,
    Subscriber,
};
use tracing_subscriber::fmt::{format::Writer, FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

const LEVEL_PREFIXES: &[&str] = &["[ERROR]", "[WARN]", "[INFO]", "[DEBUG]", "[TRACE]"];
const SUCCESS_PREFIX: &str = "[SUCCESS]";

#[derive(Clone)]
pub struct ConsoleFormatter {
    config: Arc<FormatterConfig>,
}

#[derive(Debug, Clone)]
struct FormatterConfig {
    use_ansi_colors: bool,
    include_timestamps: bool,
    include_spans: bool,
}

impl Default for ConsoleFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleFormatter {
    pub fn new() -> Self {
        Self {
            config: Arc::new(FormatterConfig {
                use_ansi_colors: true,
                include_timestamps: false,
                include_spans: false,
            }),
        }
    }

    pub fn with_ansi_colors(mut self, use_ansi_colors: bool) -> Self {
        Arc::make_mut(&mut self.config).use_ansi_colors = use_ansi_colors;
        self
    }

    pub fn with_timestamps(mut self, include_timestamps: bool) -> Self {
        Arc::make_mut(&mut self.config).include_timestamps = include_timestamps;
        self
    }

    pub fn with_spans(mut self, include_spans: bool) -> Self {
        Arc::make_mut(&mut self.config).include_spans = include_spans;
        self
    }

    fn format_level_prefix(&self, level: &Level, is_success: bool) -> String {
        if is_success {
            return if self.config.use_ansi_colors {
                format!("{}", SUCCESS_PREFIX.green().bold())
            } else {
                SUCCESS_PREFIX.to_string()
            };
        }

        let prefix = LEVEL_PREFIXES[level_to_index(level)];

        if !self.config.use_ansi_colors {
            return prefix.to_string();
        }

        match *level {
            Level::ERROR => format!("{}", prefix.red().bold()),
            Level::WARN => format!("{}", prefix.yellow().bold()),
            Level::INFO => format!("{}", prefix.blue().bold()),
            Level::DEBUG => format!("{}", prefix.cyan().bold()),
            Level::TRACE => format!("{}", prefix.magenta().bold()),
        }
    }

    fn format_timestamp(&self) -> Option<String> {
        if !self.config.include_timestamps {
            return None;
        }

        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%H:%M:%S").to_string();

        Some(if self.config.use_ansi_colors {
            format!("{}", timestamp.bright_black())
        } else {
            timestamp
        })
    }
}

impl<S, N> FormatEvent<S, N> for ConsoleFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let level = event.metadata().level();
        let mut visitor = FieldCollector::new();
        event.record(&mut visitor);

        let is_success = level == &Level::INFO && visitor.has_success_field();

        if let Some(timestamp) = self.format_timestamp() {
            write!(writer, "{} ", timestamp)?;
        }

        let prefix = self.format_level_prefix(level, is_success);
        if self.config.use_ansi_colors {
            let visual_length = get_level_visual_length(level, is_success);
            let padding = 9_usize.saturating_sub(visual_length);
            write!(writer, "{:width$}{} ", "", prefix, width = padding)?;
        } else {
            write!(writer, "{:>9} ", prefix)?;
        }

        let formatter = FieldFormatter::new(&self.config, level, is_success);
        let formatted = formatter.format_fields(&visitor.fields);
        write!(writer, "{}", formatted)?;

        writeln!(writer)
    }
}



struct FieldCollector {
    fields: SmallVec<[(String, String); 4]>,
}

impl FieldCollector {
    fn new() -> Self {
        Self {
            fields: SmallVec::new(),
        }
    }

    fn has_success_field(&self) -> bool {
        self.fields
            .iter()
            .any(|(name, value)| name == "success" && value == "true")
    }
}

impl Visit for FieldCollector {
    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.fields
            .push((field.name().to_string(), format!("{:?}", value)));
    }
}

struct FieldFormatter<'a> {
    config: &'a FormatterConfig,
    level: &'a Level,
    is_success: bool,
}

impl<'a> FieldFormatter<'a> {
    fn new(config: &'a Arc<FormatterConfig>, level: &'a Level, is_success: bool) -> Self {
        Self {
            config,
            level,
            is_success,
        }
    }

    fn format_fields(&self, fields: &[(String, String)]) -> String {
        let mut result = String::new();
        let non_message_fields: SmallVec<[&(String, String); 4]> = fields
            .iter()
            .filter(|(name, _)| name != "message" && name != "success")
            .collect();

        if let Some((_, message)) = fields.iter().find(|(name, _)| name == "message") {
            result.push_str(message);
        }

        for (field_name, value) in &non_message_fields {
            let formatted = self.format_field(field_name, value, non_message_fields.len());
            if !formatted.is_empty() {
                result.push_str(&formatted);
            }
        }

        result
    }

    fn format_field(&self, field_name: &str, value: &str, field_count: usize) -> String {
        let formatted_value = self.format_value(value);

        if field_count == 1 {
            return if self.config.use_ansi_colors {
                format!(": {}", formatted_value)
            } else {
                format!(": {}", value)
            };
        }

        if self.config.use_ansi_colors {
            self.format_colored_field(field_name, &formatted_value)
        } else {
            format!(" {}={}", field_name, value)
        }
    }

    fn format_value(&self, value: &str) -> String {
        if !self.config.use_ansi_colors {
            return value.to_string();
        }

        if self.is_success {
            return format!("{}", value.green().italic());
        }

        if is_cause_section(value) {
            return self.format_cause_section(value);
        }

        if contains_url(value) {
            self.format_with_urls(value)
        } else {
            self.format_by_level(value, false)
        }
    }



    fn format_cause_section(&self, value: &str) -> String {
        if let Some(inner) = value
            .strip_prefix("(Cause: ")
            .and_then(|s| s.strip_suffix(')'))
        {
            let formatted_inner = if contains_url(inner) {
                self.format_cause_urls(inner)
            } else {
                format!("{}", inner.red().bold())
            };

            format!(
                "{}{}{}{}",
                "(".red().bold(),
                "Cause: ".red().bold(),
                formatted_inner,
                ")".red().bold()
            )
        } else {
            format!("{}", value.red().bold())
        }
    }

    fn format_cause_urls(&self, content: &str) -> String {
        format_urls(
            content,
            |text| format!("{}", text.red().bold()),
            |url| format!("{}", url.red().bold().underline()),
        )
    }

    fn format_with_urls(&self, value: &str) -> String {
        format_urls(
            value,
            |text| self.format_by_level(text, false),
            |url| self.format_by_level(url, true),
        )
    }

    fn format_by_level(&self, value: &str, is_url: bool) -> String {
        match (*self.level, is_url) {
            (Level::ERROR, true) => format!("{}", value.red().italic().underline()),
            (Level::ERROR, false) => format!("{}", value.red().italic()),
            (Level::WARN, true) => format!("{}", value.yellow().italic().underline()),
            (Level::WARN, false) => format!("{}", value.yellow().italic()),
            (Level::INFO, true) => format!("{}", value.blue().italic().underline()),
            (Level::INFO, false) => format!("{}", value.blue().italic()),
            (Level::DEBUG, true) => format!("{}", value.cyan().italic().underline()),
            (Level::DEBUG, false) => format!("{}", value.cyan().italic()),
            (Level::TRACE, true) => format!("{}", value.magenta().italic().underline()),
            (Level::TRACE, false) => format!("{}", value.magenta().italic()),
        }
    }

    fn format_colored_field(&self, field_name: &str, formatted_value: &str) -> String {
        if self.is_success {
            return format!(" {}={}", field_name.green().italic(), formatted_value);
        }

        let colored_field = match *self.level {
            Level::ERROR => format!("{}={}", field_name.red().italic(), formatted_value),
            Level::WARN => format!("{}={}", field_name.yellow().italic(), formatted_value),
            Level::INFO => format!("{}={}", field_name.blue().italic(), formatted_value),
            Level::DEBUG => format!("{}={}", field_name.cyan().italic(), formatted_value),
            Level::TRACE => format!("{}={}", field_name.magenta().italic(), formatted_value),
        };
        format!(": {}", colored_field)
    }
}
