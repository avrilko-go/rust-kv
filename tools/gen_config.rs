use anyhow::Result;
use learn::{
    ClientConfig, ClientTlsConfig, GeneralConfig, LogConfig, RotationConfig, ServerConfig,
    ServerTlsConfig, StorageConfig,
};
use std::fs;

fn main() -> Result<()> {
    const CA_CERT: &str = include_str!("../fixtures/ca.cert");
    const SERVER_CERT: &str = include_str!("../fixtures/server.cert");
    const SERVER_KEY: &str = include_str!("../fixtures/server.key");

    let general_config = GeneralConfig {
        addr: "127.0.0.1:9527".to_string(),
    };

    let server_config = ServerConfig {
        general: general_config,
        storage: StorageConfig::SledDb("/tmp/kv_server".to_string()),
        tls: ServerTlsConfig {
            cert: SERVER_CERT.to_string(),
            key: SERVER_KEY.to_string(),
            ca: None,
        },
        log: LogConfig {
            enable_log_file: true,
            enable_jaeger: true,
            log_level: "info".to_string(),
            path: "/tmp/kv-log".to_string(),
            rotation: RotationConfig::Hourly,
        },
    };

    fs::write(
        "fixtures/server.conf",
        toml::to_string_pretty(&server_config)?,
    )?;

    let client_config = ClientConfig {
        general: GeneralConfig {
            addr: "127.0.0.1:9527".to_string(),
        },
        tls: ClientTlsConfig {
            domain: "avrilko.com".to_string(),
            identity: None,
            ca: Some(CA_CERT.into()),
        },
    };

    fs::write(
        "fixtures/client.conf",
        toml::to_string_pretty(&client_config)?,
    )?;

    Ok(())
}
