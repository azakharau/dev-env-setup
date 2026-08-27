use dev_env_setup::core::config::AppConfig;
use dev_env_setup::core::installer::OsKind;

#[test]
fn public_example_matches_the_supported_config_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config.example.toml");
    let config = AppConfig::load(path).expect("config.example.toml should parse");

    assert_eq!(config.dependencies.len(), 2);
    assert_eq!(config.required_deps().count(), 1);
    assert_eq!(config.optional_deps().count(), 1);
    assert_eq!(config.configs_for_os(OsKind::MacOS).count(), 2);
    assert_eq!(config.configs_for_os(OsKind::Linux).count(), 1);
}
