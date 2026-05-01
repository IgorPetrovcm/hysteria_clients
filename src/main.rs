use std::{collections::HashMap, path::PathBuf};

// use rand::{Rng, distributions::Alphanumeric};

use base64::{
    Engine,
    engine::{Config, general_purpose},
};
use clap::{Parser, Subcommand};
use rand::{RngExt, distr::Alphanumeric};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "hysteria_clients")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        client: String,
    },
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    Get,
    Set { key: String, value: String },
}

fn handle_config(cmd: ConfigCommands) {
    let mut config = load_app_config();

    match cmd {
        ConfigCommands::Get => {
            println!("{:#?}", config)
        }
        ConfigCommands::Set { key, value } => {
            match key.as_str() {
                "config_path" => config.hysteria_config_path = value,
                "links_path" => config.links_output_path = value,
                _ => {
                    eprintln!("Unknown key was incoming");
                    return;
                }
            }
            save_app_config(&config);
            print!("Updated config");
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Config { action } => {
            handle_config(action);
        }
        Commands::Add { client } => {
            let app_config = load_app_config();
            let hysteria_path = app_config.hysteria_config_path;

            let contents = std::fs::read_to_string(hysteria_path).expect("Read hysteria config");

            let mut hysteria_config: HysteriaConfig =
                serde_yaml::from_str(&contents).expect("Parse hysteria config");

            let password = generate_b64_password();

            hysteria_config
                .auth
                .userpass
                .insert(client.clone(), password.clone());

            let new_hysteria_config =
                serde_yaml::to_string(&hysteria_config).expect("Serialize new hysteria config");

            std::fs::write("config.yaml", new_hysteria_config).expect("Write new hysteria config");
        }
    }
}

fn generate_b64_password() -> String {
    let mut rng = rand::rng();
    let buffer: [u8; 28] = rng.random();
    return general_purpose::URL_SAFE.encode(buffer).to_string();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub hysteria_config_path: String,
    pub links_output_path: String,
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().expect("config directory");
    path.push("hysteria_clients");
    std::fs::create_dir_all(&path).unwrap();
    path.push("config.json");
    path
}

fn load_app_config() -> AppConfig {
    let path = get_config_path();
    if !path.exists() {
        let default = AppConfig {
            hysteria_config_path: "config.yml".into(),
            links_output_path: "links.txt".into(),
        };
        save_app_config(&default);
        return default;
    }

    let contents = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&contents).unwrap()
}

fn save_app_config(config: &AppConfig) {
    let path = get_config_path();
    let contents = serde_json::to_string_pretty(config).unwrap();
    std::fs::write(path, contents).unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
struct HysteriaConfig {
    auth: HysteriaAuth,
    acme: HysteriaAcme,
}

#[derive(Debug, Serialize, Deserialize)]
struct HysteriaAuth {
    #[serde(rename = "type")]
    auth_type: String,
    userpass: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HysteriaAcme {
    domains: Vec<String>,
}
