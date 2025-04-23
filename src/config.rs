use std::collections::HashMap;
use std::ops::Not;

use clap::Parser;
use serde::{Serialize, Deserialize};
use figment::{
  Figment,
  providers::{
    Serialized,
    YamlExtended,
    Env,
    Format,
  },
};
use add_ed::macros::Macro;

// Import default config
const DEFAULT_CONFIG: &str = include_str!("../default_config.yaml");

// The CLI arguments struct
// We do some fancy serde attrs to not serialize any arg not given
/// hired, the highlighting EDitor
#[derive(Parser, Debug, Serialize)]
#[clap(version, about)]
struct Args {
  /// configuration profile to use (if none given uses default)
  #[clap(long, default_value = "default")]
  #[serde(skip_serializing)]
  profile: String,
  /// path to the file to open
  #[clap(value_parser)]
  #[serde(skip_serializing_if = "Option::is_none")]
  path: Option<String>,
  /// default to printing with line numbers
  #[clap(action, short)]
  #[serde(skip_serializing_if = "<&bool>::not")]
  n: bool,
  /// default to printing in literal mode
  #[clap(action, short)]
  #[serde(skip_serializing_if = "<&bool>::not")]
  l: bool,
  /// open configuration file
  #[clap(action, long)]
  #[arg(conflicts_with("path"))]
  #[serde(skip_serializing)]
  open_config: bool,
  /// create default config file and open it
  #[clap(action, long)]
  #[arg(conflicts_with_all(["path","open_config"]))]
  #[serde(skip_serializing)]
  create_config: bool,
  /// print attributions
  #[clap(action, long)]
  #[arg(conflicts_with_all(["path", "open_config", "create_config"]))]
  #[serde(skip_serializing)]
  attributions: bool,
}

// The configuration struct
// constructed by Figment using serde::Deserialize
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
  // Path to the file to open
  #[serde(default)]
  pub path: String,
  // Default printing flags
  #[serde(default)]
  pub n: bool,
  #[serde(default)]
  pub l: bool,
  // Defined macros
  #[serde(default)]
  pub macros: HashMap<String, Macro>,
}

pub fn construct_config() -> Config {
  // First figure out platform specific config paths
  let (config_dir, config_path) = {
    let app_dirs = directories::ProjectDirs::from("se", "sidju", "hired")
      .expect("Failed to find a config directory. Is $HOME configured?")
    ;
    // Return the needed data from this block
    (app_dirs.config_dir().to_owned(), app_dirs.config_dir().join("config.yaml"))
  };
  // Parse arguments first, so we can see if we should create a default config
  let args = Args::parse();
  // If requested we print attributions and exit
  if args.attributions {
    println!();
    println!("Written by sidju, inspired by ed (by Ken Thompson)");
    println!("( See all contributors on github.com/sidju/hired and github.com/sidju/add-ed )\n");
    println!("Special thanks to the crates regex and syntect, which made this project feasible.");
    println!(
      "Attributions for the theme and all syntax definitions can be found here:\n{}",
      two_face::acknowledgement_url()
    );
    println!("Heartfelt thanks to the authors of those, and to the crates bat (and two-face) which gathered them.\n");
    std::process::exit(0);
  }
  if args.create_config {
    if config_path.exists() {
      println!(
        "There already exists a file at {}, delete it first if you wish to replace it.",
        config_path.display(),
      );
      std::process::exit(1);
    }
    else {
      // We need to first create our project folder in the config folder
      if !config_dir.is_dir() {
        std::fs::DirBuilder::new().create(config_dir)
          .expect("Error when creating config directory for hired.")
        ;
      }
      std::fs::write(&config_path, DEFAULT_CONFIG)
        .expect("Error when writing default config for hired.")
      ;
    }
  }
  let mut config: Config = Figment::new()
    // Read in config file
    .merge(YamlExtended::file(&config_path).nested())
    // Read in overrides from environment
    .merge(Env::prefixed("HIRED_").global())
    // Allow CLI arguments to override configuration
    .merge(Serialized::globals(&args))
    // Select which profile to load config from
    .select(&args.profile)
    // Convert back into config struct and verify it is valid
    .extract()
    .expect("Invalid configuration")
  ;
  // If open/create config is given we overwrite any given path with config path
  if args.open_config || args.create_config {
    config.path = config_path.into_os_string().into_string()
      .expect("Config path isn't valid unicode.")
    ;
  }

  config
}
