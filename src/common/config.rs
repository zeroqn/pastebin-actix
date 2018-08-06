#[derive(Clone, Default, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub actix: ActixConfig,
    pub postgres: PostgresConfig,
}

#[derive(Clone, Default, Deserialize)]
pub struct ServerConfig {
    pub ip: String,
    pub port: String,
}

#[derive(Clone, Default, Deserialize)]
pub struct ActixConfig {
    pub connections: usize,
}

#[derive(Clone, Default, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl Config {
    pub fn load(conf_fname: &str) -> Config {
        use std::fs::read_to_string;
        use toml;

        let config_string =
            read_to_string(conf_fname).expect(&format!("fail to read config: {}", conf_fname));
        toml::from_str(&config_string).unwrap()
    }
}
