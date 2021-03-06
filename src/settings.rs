#![allow(dead_code)]

use std;
use std::io::Write;
use std::fs::File;
use log::LevelFilter;
use clap::{App, Arg};
use config::{ConfigError, Config, File as ConfigFile, Environment};

const CONFIGURATION_FILE_NAME: &'static str = "spotme_conf";

#[derive(Debug)]
pub enum ConfigurationError {
    Help,
    Version,
    Loading(ConfigError),
    Io(::std::io::Error),
    ParseInt(::std::num::ParseIntError),
}

impl From<ConfigError> for ConfigurationError {
    fn from(e: ConfigError) -> Self {
        ConfigurationError::Loading(e)
    }
}

impl From<::std::num::ParseIntError> for ConfigurationError {
    fn from(e: ::std::num::ParseIntError) -> Self {
        ConfigurationError::ParseInt(e)
    }
}

impl From<::std::io::Error> for ConfigurationError {
    fn from(e: ::std::io::Error) -> Self {
        ConfigurationError::Io(e)
    }
}

type Result<T> = std::result::Result<T, ConfigurationError>;

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(default)]
pub struct Server {
    pub mongo_uri: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct Settings {
    pub loglevel: String,
    pub server: Server,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            loglevel: "info".to_string(),
            server: Server {
                mongo_uri: String::new()
            }
        }
    }
}

impl Settings {
    pub fn level_filter(&self) -> LevelFilter {
        match self.loglevel.to_lowercase().as_str() {
            "off" => LevelFilter::Off,
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Off,
        }
    }

    pub fn load() -> Result<Self> {
        use std::path::Path;
        let cli_app = create_command_line_app();
        let matches = cli_app.get_matches();
        let default = Config::try_from(&Settings::default())?;
        let mut conf = Config::new();

        conf.merge(default)?;

        if let Some(path) = matches.value_of("config-file") {
            let p = Path::new(path);
            conf.merge(ConfigFile::from(p).required(true))?;
        } else {
            conf.merge(ConfigFile::with_name(CONFIGURATION_FILE_NAME).required(false))?;
        }

        conf.merge(Environment::with_prefix("spotme").separator("_"))?;

        let mut settings: Settings = conf.try_into()?;

        // Apply command line arg

        if let Some(level) = matches.value_of("log-level") {
            settings.loglevel = level.to_string();
        };

        if let Some(uri) = matches.value_of("mongo-uri") {
            settings.server.mongo_uri = uri.to_string();
        }

        if let Some(config_path) = matches.value_of("save-config") {
            let mut file_path = Path::new(config_path).to_owned();

            file_path.push(CONFIGURATION_FILE_NAME);

            match matches.value_of("format").unwrap_or_else(|| "TOML") {
                "TOML" => {
                    use toml;

                    if let Ok(pretty) = toml::to_string_pretty(&settings) {
                        file_path.set_extension("toml");
                        let mut file = File::create(file_path)?;
                        file.write_all(pretty.as_bytes())?;
                    }
                }
                "YAML" => {
                    use serde_yaml;

                    if let Ok(pretty) = serde_yaml::to_string(&settings) {
                        file_path.set_extension("yaml");
                        let mut file = File::create(file_path)?;
                        file.write_all(pretty.as_bytes())?;
                    }
                }
                "JSON" => {
                    use serde_json;

                    if let Ok(pretty) = serde_json::to_string_pretty(&settings) {
                        file_path.set_extension("json");
                        let mut file = File::create(file_path)?;
                        file.write_all(pretty.as_bytes())?;
                    }
                }
                wrong => {
                    println!("Specified configuration format is invalid {}", wrong);
                    ::std::process::exit(1);
                }
            }
        }

        if matches.is_present("show-config") {
            use toml::to_string_pretty;

            if let Ok(pretty) = to_string_pretty(&settings) {
                println!("------------------------SPOTME CONFIGURATION------------------------\n{}\n---------------------------------------------------------------------", pretty);
            }
        }

        Ok(settings)
    }
}

fn create_command_line_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .author("Seb Aubin - Devolutions")
        .version(concat!(crate_version!(), "\n"))
        .version_short("v")
        .about("A simple identity server")
        .arg(Arg::with_name("config-file")
            .short("c")
            .long("config")
            .value_name("CONFIGFILE")
            .help("Path of a custom configuration file")
            .takes_value(true)
            .empty_values(false)
        )
        .arg(Arg::with_name("log-level")
            .short("l")
            .long("level")
            .value_name("LOGLEVEL")
            .help("Verbosity level of the logger")
            .takes_value(true)
            .possible_values(&["off", "error", "warn", "info", "debug", "trace"])
            .empty_values(false)
        )
        .arg(Arg::with_name("mongo-uri")
            .short("u")
            .long("uri")
            .value_name("MONGO_URI")
            .help("Uri of the mongo server")
            .takes_value(true)
            .empty_values(false)
        )
        .arg(Arg::with_name("sav e-config")
            .long("save-config")
            .value_name("PATH")
            .help("Save the current config at the specified directory (default file format is TOML, see `format` for more)")
            .takes_value(true)
            .empty_values(false)
        )
        .arg(Arg::with_name("format")
            .long("format")
            .value_name("FORMAT")
            .help("Use with --save-config: Specifies which format will be used to save configurations")
            .possible_values(&["TOML", "YAML", "JSON"])
            .default_value("TOML")
            .takes_value(true)
            .empty_values(false)
        )
        .arg(Arg::with_name("show-config")
            .long("show-config")
            .help("Show the current config before startup")
            .takes_value(false)
        )
}