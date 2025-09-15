use criterion::{
  Criterion,
  criterion_group,
  criterion_main,
};
use std::hint::black_box;

use std::{
  collections::HashMap,
  io::{
    BufRead,
    BufReader,
    Cursor,
    Read,
  },
};

// Using standard allocator - no global allocator declaration

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
}

const SAMPLE_INI: &[u8] = include_bytes!("sample.ini");

fn bench_ini_parsing_std(c: &mut Criterion) {
  c.bench_function("ini_parse_std", |b| {
    b.iter(|| {
      let mut parser = IniParser::new();
      let cursor = Cursor::new(SAMPLE_INI);
      parser.parse(black_box(cursor)).unwrap();
      black_box(parser.get_global_property("debug"));
    });
  });
}

criterion_group!(benches, bench_ini_parsing_std);
criterion_main!(benches);
