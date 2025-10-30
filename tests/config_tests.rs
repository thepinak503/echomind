use echomind::config::{ApiConfig, Config, Defaults};

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.api.provider, "chat");
    assert_eq!(config.api.model, "gpt-3.5-turbo");
    assert_eq!(config.api.timeout, 30);
    assert_eq!(config.defaults.temperature, 0.7);
    assert_eq!(config.defaults.coder_mode, false);
}

#[test]
fn test_config_serialization() {
    let config = Config {
        api: ApiConfig {
            provider: "openai".to_string(),
            api_key: Some("test-key".to_string()),
            endpoint: None,
            model: "gpt-4".to_string(),
            timeout: 60,
        },
        defaults: Defaults {
            temperature: 0.5,
            max_tokens: Some(1000),
            coder_mode: true,
            stream: false,
        },
    };

    let toml_str = toml::to_string(&config).unwrap();
    assert!(toml_str.contains("provider = \"openai\""));
    assert!(toml_str.contains("model = \"gpt-4\""));
    assert!(toml_str.contains("temperature = 0.5"));
}

#[test]
fn test_config_deserialization() {
    let toml_str = r#"
        [api]
        provider = "claude"
        api_key = "sk-test123"
        model = "claude-3-opus"
        timeout = 45

        [defaults]
        temperature = 0.8
        max_tokens = 2000
        coder_mode = true
        stream = true
    "#;

    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.api.provider, "claude");
    assert_eq!(config.api.api_key, Some("sk-test123".to_string()));
    assert_eq!(config.api.model, "claude-3-opus");
    assert_eq!(config.api.timeout, 45);
    assert_eq!(config.defaults.temperature, 0.8);
    assert_eq!(config.defaults.max_tokens, Some(2000));
    assert_eq!(config.defaults.coder_mode, true);
    assert_eq!(config.defaults.stream, true);
}

#[test]
fn test_partial_config() {
    // Test that defaults are applied when fields are missing
    let toml_str = r#"
        [api]
        provider = "ollama"
    "#;

    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.api.provider, "ollama");
    assert_eq!(config.api.model, "gpt-3.5-turbo"); // Should use default
    assert_eq!(config.defaults.temperature, 0.7); // Should use default
}
