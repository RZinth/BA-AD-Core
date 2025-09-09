use crate::utils::{
    contains_url, format_urls, get_level_visual_length, level_to_index,
};

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
const CAUSE_PREFIX: &str = "[CAUSE]";

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

    fn write_timestamp(&self, writer: &mut Writer<'_>) -> fmt::Result {
        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%H:%M:%S");

        if self.config.use_ansi_colors {
            write!(writer, "{}", timestamp.to_string().bright_black())
        } else {
            write!(writer, "{}", timestamp)
        }
    }

    fn write_level_prefix(&self, writer: &mut Writer<'_>, level: &Level, is_success: bool) -> fmt::Result {
        if is_success {
            if self.config.use_ansi_colors {
                let visual_length = get_level_visual_length(level, is_success);
                let padding = 9_usize.saturating_sub(visual_length);
                write!(writer, "{:width$}{}", "", SUCCESS_PREFIX.green().bold(), width = padding)
            } else {
                write!(writer, "{:>9}", SUCCESS_PREFIX)
            }
        } else {
            let prefix = LEVEL_PREFIXES[level_to_index(level)];

            if self.config.use_ansi_colors {
                let visual_length = get_level_visual_length(level, is_success);
                let padding = 9_usize.saturating_sub(visual_length);
                let colored_prefix = match *level {
                    Level::ERROR => format!("{}", prefix.red().bold()),
                    Level::WARN => format!("{}", prefix.yellow().bold()),
                    Level::INFO => format!("{}", prefix.blue().bold()),
                    Level::DEBUG => format!("{}", prefix.cyan().bold()),
                    Level::TRACE => format!("{}", prefix.magenta().bold()),
                };
                write!(writer, "{:width$}{}", "", colored_prefix, width = padding)
            } else {
                write!(writer, "{:>9}", prefix)
            }
        }
    }

    fn write_simple_message(
        &self,
        writer: &mut Writer<'_>,
        level: &Level,
        is_success: bool,
        fields: &[(String, String)],
    ) -> fmt::Result {
        // Write level prefix directly
        self.write_level_prefix(writer, level, is_success)?;
        write!(writer, " ")?;

        // Write message directly (we know it's the only field)
        if let Some((_, message)) = fields.first() {
            write!(writer, "{}", message)?;
        }

        Ok(())
    }

    fn write_cause_line(&self, writer: &mut Writer<'_>, cause_value: &str) -> fmt::Result {
        // Write timestamp if enabled
        if self.config.include_timestamps {
            self.write_timestamp(writer)?;
            write!(writer, " ")?;
        }

        // Write cause prefix
        if self.config.use_ansi_colors {
            let visual_length = 7; // "[CAUSE]" length
            let padding = 9_usize.saturating_sub(visual_length);
            write!(writer, "{:width$}{} ", "", CAUSE_PREFIX.red().bold(), width = padding)?;
        } else {
            write!(writer, "{:>9} ", CAUSE_PREFIX)?;
        }

        // Write cause value with appropriate formatting
        if self.config.use_ansi_colors {
            if contains_url(cause_value) {
                let formatted = format_urls(
                    cause_value,
                    |text| format!("{}", text.red().italic()),
                    |url| format!("{}", url.red().italic().underline()),
                );
                write!(writer, "{}", formatted)?;
            } else {
                write!(writer, "{}", cause_value.red().italic())?;
            }
        } else {
            write!(writer, "{}", cause_value)?;
        }

        writeln!(writer)
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

        if visitor.is_simple_message() && !self.config.include_timestamps {
            self.write_simple_message(&mut writer, level, is_success, &visitor.fields)?;
            return writeln!(writer);
        }

        if self.config.include_timestamps {
            self.write_timestamp(&mut writer)?;
            write!(writer, " ")?;
        }

        self.write_level_prefix(&mut writer, level, is_success)?;
        write!(writer, " ")?;

        let formatter = FieldFormatter::new(&self.config, level, is_success);
        formatter.write_fields(&mut writer, &visitor.fields)?;

        writeln!(writer)?;

        if let Some(cause_value) = visitor.get_cause_value() {
            self.write_cause_line(&mut writer, cause_value)?;
        }

        Ok(())
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

    fn is_simple_message(&self) -> bool {
        self.fields.len() == 1 && 
        self.fields.first().map(|(name, _)| name == "message").unwrap_or(false)
    }

    fn get_cause_value(&self) -> Option<&str> {
        self.fields.iter()
            .find(|(name, _)| name == "cause")
            .map(|(_, value)| value.as_str())
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

    fn write_fields(&self, writer: &mut Writer<'_>, fields: &[(String, String)]) -> fmt::Result {
        let non_message_fields: SmallVec<[&(String, String); 4]> = fields
            .iter()
            .filter(|(name, _)| name != "message" && name != "success" && name != "cause")
            .collect();

        if let Some((_, message)) = fields.iter().find(|(name, _)| name == "message") {
            write!(writer, "{}", message)?;
        }

        for (i, (field_name, value)) in non_message_fields.iter().enumerate() {
            self.write_field(writer, field_name, value, non_message_fields.len(), i == 0)?;
        }

        Ok(())
    }

    fn write_field(
        &self,
        writer: &mut Writer<'_>,
        field_name: &str,
        value: &str,
        field_count: usize,
        is_first: bool,
    ) -> fmt::Result {
        if field_count == 1 {
            write!(writer, ": ")?;
            if self.config.use_ansi_colors {
                self.write_colored_value(writer, value)?;
            } else {
                write!(writer, "{}", value)?;
            }
            return Ok(());
        }

        let separator = if is_first { ": " } else { ", " };
        write!(writer, "{}", separator)?;

        if self.config.use_ansi_colors {
            self.write_colored_field(writer, field_name, value)?;
        } else {
            write!(writer, "{}={}", field_name, value)?;
        }

        Ok(())
    }

    fn write_colored_value(&self, writer: &mut Writer<'_>, value: &str) -> fmt::Result {
        if !self.config.use_ansi_colors {
            return write!(writer, "{}", value);
        }

        if self.is_success {
            write!(writer, "{}", value.green().italic())?;
            return Ok(());
        }

        if !contains_url(value) {
            match *self.level {
                Level::ERROR => write!(writer, "{}", value.red().italic()),
                Level::WARN => write!(writer, "{}", value.yellow().italic()),
                Level::INFO => write!(writer, "{}", value.blue().italic()),
                Level::DEBUG => write!(writer, "{}", value.cyan().italic()),
                Level::TRACE => write!(writer, "{}", value.magenta().italic()),
            }
        } else {
            let formatted = self.format_with_urls(value);
            write!(writer, "{}", formatted)
        }
    }

    fn write_colored_field(&self, writer: &mut Writer<'_>, field_name: &str, value: &str) -> fmt::Result {
        if self.is_success {
            write!(writer, "{}=", field_name.green().italic())?;
            self.write_colored_value(writer, value)?;
            return Ok(());
        }

        let colored_field_name = match *self.level {
            Level::ERROR => format!("{}", field_name.red().italic()),
            Level::WARN => format!("{}", field_name.yellow().italic()),
            Level::INFO => format!("{}", field_name.blue().italic()),
            Level::DEBUG => format!("{}", field_name.cyan().italic()),
            Level::TRACE => format!("{}", field_name.magenta().italic()),
        };

        write!(writer, "{}=", colored_field_name)?;
        self.write_colored_value(writer, value)?;

        Ok(())
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

}
