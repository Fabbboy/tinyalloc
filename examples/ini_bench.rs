use tinyalloc::TinyAlloc;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Cursor, Read};
use std::time::{Duration, Instant};

//#[global_allocator]
//static GLOBAL_ALLOCATOR: TinyAlloc = TinyAlloc;

#[derive(Debug, Clone, PartialEq)]
struct IniSection {
    name: String,
    properties: HashMap<String, String>,
}

impl IniSection {
    fn new(name: String) -> Self {
        Self {
            name,
            properties: HashMap::new(),
        }
    }

    fn add_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
}

#[derive(Debug, Default)]
struct IniParser {
    sections: HashMap<String, IniSection>,
    global_properties: HashMap<String, String>,
}

impl IniParser {
    fn new() -> Self {
        Self::default()
    }

    fn clear(&mut self) {
        self.sections.clear();
        self.global_properties.clear();
    }

    fn parse_line(&mut self, line: &str, current_section: &mut Option<String>) -> Result<(), String> {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            return Ok(());
        }

        if let Some(section_name) = Self::parse_section_header(trimmed) {
            let section_name = section_name.to_string();
            self.sections.entry(section_name.clone())
                .or_insert_with(|| IniSection::new(section_name.clone()));
            *current_section = Some(section_name);
            return Ok(());
        }

        if let Some((key, value)) = Self::parse_property(trimmed) {
            match current_section {
                Some(section_name) => {
                    if let Some(section) = self.sections.get_mut(section_name) {
                        section.add_property(key.to_string(), value.to_string());
                    }
                }
                None => {
                    self.global_properties.insert(key.to_string(), value.to_string());
                }
            }
            return Ok(());
        }

        Err(format!("Invalid INI line: {}", trimmed))
    }

    fn parse_section_header(line: &str) -> Option<&str> {
        if line.starts_with('[') && line.ends_with(']') && line.len() > 2 {
            Some(&line[1..line.len()-1])
        } else {
            None
        }
    }

    fn parse_property(line: &str) -> Option<(&str, &str)> {
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim();
            let value = line[eq_pos + 1..].trim();
            if !key.is_empty() {
                Some((key, value))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn parse<R: Read>(&mut self, reader: R) -> Result<(), String> {
        let buf_reader = BufReader::new(reader);
        let mut current_section: Option<String> = None;

        for (line_num, line_result) in buf_reader.lines().enumerate() {
            let line = line_result
                .map_err(|e| format!("IO error at line {}: {}", line_num + 1, e))?;

            if let Err(parse_error) = self.parse_line(&line, &mut current_section) {
                return Err(format!("Parse error at line {}: {}", line_num + 1, parse_error));
            }
        }

        Ok(())
    }

    fn get_section(&self, name: &str) -> Option<&IniSection> {
        self.sections.get(name)
    }

    fn get_global_property(&self, key: &str) -> Option<&String> {
        self.global_properties.get(key)
    }
}

const BENCHMARK_INI_CONTENT: &str = r#"
# Global configuration
debug = true
log_level = info
max_connections = 1000
timeout = 30
version = 2.1.0

[database]
# Database configuration
type = postgresql
host = localhost
port = 5432
name = production_db
user = admin
password = secret123
ssl_mode = require
connection_pool_size = 25
query_timeout = 10

[cache]
# Cache configuration
enabled = true
type = redis
host = cache.example.com
port = 6379
ttl = 3600
max_memory = 512MB
eviction_policy = allkeys-lru

[logging]
# Logging configuration
format = json
level = warn
output = /var/log/app.log
rotate = true
max_size = 100MB
max_files = 10

[security]
# Security settings
enable_https = true
ssl_cert = /etc/ssl/certs/app.crt
ssl_key = /etc/ssl/private/app.key
api_key_header = X-API-Key
rate_limit = 100
rate_limit_window = 60

[monitoring]
# Monitoring and metrics
enabled = true
endpoint = /metrics
port = 9090
collect_gc_stats = true
collect_memory_stats = true
collect_cpu_stats = true

[feature_flags]
# Feature toggles
new_ui = true
beta_api = false
advanced_search = true
real_time_updates = true
analytics = true

[email]
# Email configuration
smtp_host = smtp.gmail.com
smtp_port = 587
smtp_user = notifications@example.com
smtp_password = email_secret
from_address = noreply@example.com
reply_to = support@example.com

[backup]
# Backup settings
enabled = true
schedule = 0 2 * * *
retention_days = 30
destination = s3://backups/database
compression = gzip
encryption = true

[api]
# API configuration
base_url = https://api.example.com
version = v1
timeout = 15
retries = 3
retry_delay = 1000
max_payload_size = 10MB
"#;

const ITERATIONS: usize = 10_000;

fn benchmark_parsing() -> Duration {
    let mut parser = IniParser::new();
    let start = Instant::now();

    for _ in 0..ITERATIONS {
        parser.clear();
        let cursor = Cursor::new(BENCHMARK_INI_CONTENT);

        match parser.parse(cursor) {
            Ok(()) => {
                let _ = parser.get_global_property("debug");
                let _ = parser.get_section("database");
                let _ = parser.get_section("cache");
            }
            Err(e) => {
                eprintln!("Parse error: {}", e);
                break;
            }
        }
    }

    start.elapsed()
}

fn print_statistics(parser: &IniParser) {
    println!("Parsed INI Statistics:");
    println!("Global properties: {}", parser.global_properties.len());
    println!("Sections: {}", parser.sections.len());

    let total_properties: usize = parser.sections.values()
        .map(|section| section.properties.len())
        .sum();
    println!("Total properties in sections: {}", total_properties);
    println!("Total properties: {}", parser.global_properties.len() + total_properties);
}

fn main() {
    println!("INI Parser Benchmark");
    println!("==================");

    // Parse once to show what we're working with
    let mut parser = IniParser::new();
    let cursor = Cursor::new(BENCHMARK_INI_CONTENT);

    match parser.parse(cursor) {
        Ok(()) => {
            print_statistics(&parser);
        }
        Err(e) => {
            eprintln!("Initial parse failed: {}", e);
            std::process::exit(1);
        }
    }

    println!("\nRunning benchmark with {} iterations...", ITERATIONS);
    let duration = benchmark_parsing();

    let total_ms = duration.as_millis();
    let avg_us = duration.as_micros() / ITERATIONS as u128;
    let ops_per_sec = (ITERATIONS as f64 / duration.as_secs_f64()) as u64;

    println!("\nBenchmark Results:");
    println!("Total time: {}ms", total_ms);
    println!("Average per iteration: {}μs", avg_us);
    println!("Operations per second: {}", ops_per_sec);

    println!("\nTo profile with perf:");
    println!("perf record --call-graph=dwarf ./target/release/examples/ini_benchmark");
    println!("perf report");
}