use cbindgen::{Config, Language};

extern crate cbindgen;

fn main() {
  let crate_path = env!("CARGO_MANIFEST_DIR");
  let output_file = format!("{}/tinyalloc.h", crate_path);

  let config = Config {
    language: Language::C,
    ..Default::default()
  };

  cbindgen::generate_with_config(crate_path, config)
    .expect("Unable to generate bindings")
    .write_to_file(output_file);
}
