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

fn bench_ini_parsing_tiny(c: &mut Criterion) {
  let mut group = c.benchmark_group("ini_parsing_tiny");

  group.bench_function("parse_fresh", |b| {
    b.iter(|| {
      let mut parser = IniParser::new();
      let cursor = Cursor::new(SAMPLE_INI);
      parser.parse(black_box(cursor)).unwrap();

      // Access some data to ensure it's not optimized away
      let _ = black_box(parser.get_global_property("debug"));
      let _ = black_box(parser.get_section("database"));
    });
  });

  group.bench_function("parse_reuse", |b| {
    let mut parser = IniParser::new();
    b.iter(|| {
      parser.clear();
      let cursor = Cursor::new(SAMPLE_INI);
      parser.parse(black_box(cursor)).unwrap();

      // Access some data to ensure it's not optimized away
      let _ = black_box(parser.get_global_property("debug"));
      let _ = black_box(parser.get_section("database"));
    });
  });

  group.finish();
}

fn bench_property_access_tiny(c: &mut Criterion) {
  let mut parser = IniParser::new();
  let cursor = Cursor::new(SAMPLE_INI);
  parser.parse(cursor).unwrap();

  c.bench_function("property_access_global_tiny", |b| {
    b.iter(|| {
      black_box(parser.get_global_property("debug"));
      black_box(parser.get_global_property("log_level"));
      black_box(parser.get_global_property("max_connections"));
    });
  });

  c.bench_function("property_access_section_tiny", |b| {
    b.iter(|| {
      if let Some(db_section) = parser.get_section("database") {
        black_box(db_section.get_property("host"));
        black_box(db_section.get_property("port"));
        black_box(db_section.get_property("user"));
      }
    });
  });
}

fn bench_line_parsing_tiny(c: &mut Criterion) {
  let mut parser = IniParser::new();
  let mut current_section = None;

  c.bench_function("parse_section_header_tiny", |b| {
    b.iter(|| {
      black_box(IniParser::parse_section_header("[database]"));
      black_box(IniParser::parse_section_header("[cache]"));
      black_box(IniParser::parse_section_header("[logging]"));
    });
  });

  c.bench_function("parse_property_tiny", |b| {
    b.iter(|| {
      black_box(IniParser::parse_property("host = localhost"));
      black_box(IniParser::parse_property("port=5432"));
      black_box(IniParser::parse_property("enabled = true"));
    });
  });

  c.bench_function("parse_line_complete_tiny", |b| {
    b.iter(|| {
      parser
        .parse_line(black_box("host = localhost"), &mut current_section)
        .unwrap();
      parser
        .parse_line(black_box("[section]"), &mut current_section)
        .unwrap();
      parser
        .parse_line(black_box("# comment"), &mut current_section)
        .unwrap();
    });
  });
}

criterion_group!(
  benches,
  bench_ini_parsing_tiny,
  bench_property_access_tiny,
  bench_line_parsing_tiny
);
criterion_main!(benches);
