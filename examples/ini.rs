use std::{
  collections::HashMap,
  io::{
    self,
    BufRead,
    BufReader,
    Read,
  },
};
use tinyalloc::TinyAlloc;

#[global_allocator]
static GLOBAL_ALLOCATOR: TinyAlloc = TinyAlloc;

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

  fn parse_line(
    &mut self,
    line: &str,
    current_section: &mut Option<String>,
  ) -> Result<(), String> {
    let trimmed = line.trim();

    if trimmed.is_empty()
      || trimmed.starts_with('#')
      || trimmed.starts_with(';')
    {
      return Ok(());
    }

    if let Some(section_name) = Self::parse_section_header(trimmed) {
      let section_name = section_name.to_string();
      self
        .sections
        .entry(section_name.clone())
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
          self
            .global_properties
            .insert(key.to_string(), value.to_string());
        }
      }
      return Ok(());
    }

    Err(format!("Invalid INI line: {}", trimmed))
  }

  fn parse_section_header(line: &str) -> Option<&str> {
    if line.starts_with('[') && line.ends_with(']') && line.len() > 2 {
      Some(&line[1..line.len() - 1])
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
        return Err(format!(
          "Parse error at line {}: {}",
          line_num + 1,
          parse_error
        ));
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

  fn print_parsed_data(&self) {
    if !self.global_properties.is_empty() {
      println!("Global properties:");
      for (key, value) in &self.global_properties {
        println!("  {} = {}", key, value);
      }
      println!();
    }

    for (section_name, section) in &self.sections {
      println!("[{}]", section_name);
      for (key, value) in &section.properties {
        println!("  {} = {}", key, value);
      }
      println!();
    }
  }
}

fn main() -> io::Result<()> {
  println!("INI Parser - Enter INI content, press Ctrl+D when finished:");
  println!("Example format:");
  println!("global_key = global_value");
  println!("[section1]");
  println!("key1 = value1");
  println!("key2 = value2");
  println!();

  let mut parser = IniParser::new();

  match parser.parse(io::stdin()) {
    Ok(()) => {
      println!("Successfully parsed INI data:");
      println!("{}", "=".repeat(40));
      parser.print_parsed_data();

      println!("Query examples:");
      if let Some(value) = parser.get_global_property("global_key") {
        println!("Global property 'global_key': {}", value);
      }

      if let Some(section) = parser.get_section("section1") {
        if let Some(value) = section.get_property("key1") {
          println!("Section 'section1', property 'key1': {}", value);
        }
      }
    }
    Err(error) => {
      eprintln!("Parse error: {}", error);
      std::process::exit(1);
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Cursor;

  #[test]
  fn test_parse_section_header() {
    assert_eq!(
      IniParser::parse_section_header("[section]"),
      Some("section")
    );
    assert_eq!(
      IniParser::parse_section_header("[section with spaces]"),
      Some("section with spaces")
    );
    assert_eq!(IniParser::parse_section_header("[]"), None);
    assert_eq!(IniParser::parse_section_header("["), None);
    assert_eq!(IniParser::parse_section_header("section]"), None);
    assert_eq!(IniParser::parse_section_header("not_section"), None);
  }

  #[test]
  fn test_parse_property() {
    assert_eq!(
      IniParser::parse_property("key=value"),
      Some(("key", "value"))
    );
    assert_eq!(
      IniParser::parse_property("key = value"),
      Some(("key", "value"))
    );
    assert_eq!(IniParser::parse_property("key="), Some(("key", "")));
    assert_eq!(IniParser::parse_property("=value"), None);
    assert_eq!(IniParser::parse_property("no_equals"), None);
    assert_eq!(
      IniParser::parse_property("multiple=equals=signs"),
      Some(("multiple", "equals=signs"))
    );
  }

  #[test]
  fn test_full_ini_parsing() {
    let ini_content = r#"
# This is a comment
global_prop = global_value

[database]
host = localhost
port = 5432
; This is another comment
user = admin

[cache]
enabled = true
ttl = 3600
"#;

    let cursor = Cursor::new(ini_content);
    let mut parser = IniParser::new();

    assert!(parser.parse(cursor).is_ok());

    assert_eq!(
      parser.get_global_property("global_prop"),
      Some(&"global_value".to_string())
    );

    let db_section = parser.get_section("database").unwrap();
    assert_eq!(
      db_section.get_property("host"),
      Some(&"localhost".to_string())
    );
    assert_eq!(db_section.get_property("port"), Some(&"5432".to_string()));
    assert_eq!(db_section.get_property("user"), Some(&"admin".to_string()));

    let cache_section = parser.get_section("cache").unwrap();
    assert_eq!(
      cache_section.get_property("enabled"),
      Some(&"true".to_string())
    );
    assert_eq!(cache_section.get_property("ttl"), Some(&"3600".to_string()));
  }

  #[test]
  fn test_empty_input() {
    let cursor = Cursor::new("");
    let mut parser = IniParser::new();

    assert!(parser.parse(cursor).is_ok());
    assert!(parser.global_properties.is_empty());
    assert!(parser.sections.is_empty());
  }

  #[test]
  fn test_invalid_line() {
    let ini_content = "invalid line without equals or brackets";
    let cursor = Cursor::new(ini_content);
    let mut parser = IniParser::new();

    assert!(parser.parse(cursor).is_err());
  }
}
