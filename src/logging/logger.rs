use std::sync::Arc;

use chrono::{DateTime, Utc};
use colored::Colorize;

#[derive(Debug, Clone, Copy)]
pub enum LoggingEvent {
    APPSTARTED,
    DOWNLOADSTARTED,
    APPCRASHED,
    TORRENTFOUND,
    PEERCONNECTED,
}

impl LoggingEvent {
    pub fn to_str(&mut self) -> &str {
        match self {
            LoggingEvent::APPCRASHED => "APP_CRASHED",
            LoggingEvent::PEERCONNECTED => "PEER_CONNECTED",
            LoggingEvent::DOWNLOADSTARTED => "DOWNLOAD_STARTED",
            LoggingEvent::TORRENTFOUND => "TORRENT_FOUND",
            LoggingEvent::APPSTARTED => "APP_STARTED",
        }
    }

    pub fn colored_str(&mut self) -> String {
        match self {
            LoggingEvent::APPCRASHED => self.to_str().bright_red().to_string(),
            LoggingEvent::PEERCONNECTED => self.to_str().bright_magenta().to_string(),
            LoggingEvent::DOWNLOADSTARTED => self.to_str().bright_blue().to_string(),
            LoggingEvent::TORRENTFOUND => self.to_str().bright_green().to_string(),
            LoggingEvent::APPSTARTED => self.to_str().bright_yellow().to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    INFO,
    TRACE,
    WARN,
    DEBUG,
    ERROR,
}

impl LogLevel {
    pub fn to_str(&self) -> &str {
        match self {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::ERROR => "ERROR",
            LogLevel::WARN => "WARN",
            LogLevel::TRACE => "TRACE",
            LogLevel::INFO => "INFO",
        }
    }

    pub fn colored_str(&self) -> String {
        // Colors the strings , used the Colorized crate for this
        match self {
            LogLevel::DEBUG => self.to_str().purple().to_string(),
            LogLevel::ERROR => self.to_str().red().bold().to_string(),
            LogLevel::WARN => self.to_str().yellow().to_string(),
            LogLevel::TRACE => self.to_str().bright_green().to_string(),
            LogLevel::INFO => self.to_str().blue().to_string(),
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            LogLevel::ERROR => 0,
            LogLevel::WARN => 5,
            LogLevel::INFO => 10,
            LogLevel::DEBUG => 15,
            LogLevel::TRACE => 20,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    pub event: LoggingEvent,
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl Log {
    pub fn new(event: LoggingEvent, level: LogLevel, message: String) -> Self {
        Self {
            event,
            level,
            message,
            timestamp: Utc::now(),
        }
    }

    pub fn format(&mut self) -> String {
        let timestamp = format!("[{}] ", self.timestamp.format("%Y-%m-%d | %H:%M:%S UTC"));
        // let level = self.level.do
        format!(
            "{} {} {}",
            timestamp,
            self.level.colored_str(),
            self.event.to_str()
        )
    }
}

pub struct Logger {
    pub level: Arc<LogLevel>,
    pub timestamp: DateTime<Utc>,
    pub logs: Vec<Log>,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level: Arc::new(level),
            timestamp: Utc::now(),
            logs: Vec::new(),
        }
    }

    pub fn log(&mut self, event: LoggingEvent, level: LogLevel, message: String) -> String {
        let mut log = Log {
            event,
            level,
            message,
            timestamp: Utc::now(),
        };

        self.logs.push(log.clone());
        return log.format();
    }

    pub fn info(&mut self, event: LoggingEvent, message: String) {
        self.log(event, LogLevel::INFO, message);
    }
    pub fn error(&mut self, event: LoggingEvent, message: String) {
        self.log(event, LogLevel::ERROR, message);
    }

    pub fn debug(&mut self, event: LoggingEvent, message: String) {
        self.log(event, LogLevel::DEBUG, message);
    }

    pub fn trace(&mut self, event: LoggingEvent, message: String) {
        self.log(event, LogLevel::TRACE, message);
    }
    pub fn warn(&mut self, event: LoggingEvent, message: String) {
        self.log(event, LogLevel::WARN, message);
    }
}
