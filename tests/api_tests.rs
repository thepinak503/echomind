use echomind::api::{ChatRequest, Message, Provider};

#[test]
fn test_provider_from_string() {
    assert!(matches!(
        Provider::from_string("chat").unwrap(),
        Provider::Chat
    ));
    assert!(matches!(
        Provider::from_string("chatanywhere").unwrap(),
        Provider::ChatAnywhere
    ));
    assert!(matches!(
        Provider::from_string("openai").unwrap(),
        Provider::OpenAI
    ));
    assert!(matches!(
        Provider::from_string("claude").unwrap(),
        Provider::Claude
    ));
    assert!(matches!(
        Provider::from_string("ollama").unwrap(),
        Provider::Ollama
    ));
    assert!(matches!(
        Provider::from_string("grok").unwrap(),
        Provider::Grok
    ));
    assert!(matches!(
        Provider::from_string("mistral").unwrap(),
        Provider::Mistral
    ));
    assert!(matches!(
        Provider::from_string("cohere").unwrap(),
        Provider::Cohere
    ));

    // Test case insensitivity
    assert!(matches!(
        Provider::from_string("OPENAI").unwrap(),
        Provider::OpenAI
    ));
    assert!(matches!(
        Provider::from_string("Chat").unwrap(),
        Provider::Chat
    ));

    // Test custom endpoint
    let custom = Provider::from_string("https://custom.api.com/v1/chat").unwrap();
    assert!(matches!(custom, Provider::Custom(_)));

    // Test invalid provider
    assert!(Provider::from_string("invalid").is_err());
}

#[test]
fn test_provider_endpoints() {
    assert_eq!(
        Provider::Chat.endpoint(),
        "https://ch.at/v1/chat/completions"
    );
    assert_eq!(
        Provider::ChatAnywhere.endpoint(),
        "https://api.chatanywhere.tech/v1/chat/completions"
    );
    assert_eq!(
        Provider::OpenAI.endpoint(),
        "https://api.openai.com/v1/chat/completions"
    );
    assert_eq!(
        Provider::Ollama.endpoint(),
        "http://localhost:11434/api/chat"
    );
    assert_eq!(
        Provider::Grok.endpoint(),
        "https://api.x.ai/v1/chat/completions"
    );
    assert_eq!(
        Provider::Mistral.endpoint(),
        "https://api.mistral.ai/v1/chat/completions"
    );
    assert_eq!(
        Provider::Cohere.endpoint(),
        "https://api.cohere.ai/v1/chat"
    );
}

#[test]
fn test_provider_requires_api_key() {
    assert!(!Provider::Chat.requires_api_key());
    assert!(Provider::ChatAnywhere.requires_api_key());
    assert!(Provider::OpenAI.requires_api_key());
    assert!(Provider::Claude.requires_api_key());
    assert!(Provider::Grok.requires_api_key());
    assert!(Provider::Mistral.requires_api_key());
    assert!(Provider::Cohere.requires_api_key());
    assert!(!Provider::Ollama.requires_api_key());
}

#[test]
fn test_message_creation() {
    let msg = Message::text(
        "user".to_string(),
        "Hello, AI!".to_string(),
    );
    assert_eq!(msg.role, "user");
    assert_eq!(msg.get_text(), Some("Hello, AI!"));
}

#[test]
fn test_chat_request_serialization() {
    let request = ChatRequest {
        messages: vec![Message::text(
            "user".to_string(),
            "Test message".to_string(),
        )],
        model: Some("gpt-4".to_string()),
        temperature: Some(0.7),
        max_tokens: Some(1000),
        stream: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("Test message"));
    assert!(json.contains("gpt-4"));
    assert!(json.contains("0.7"));
}

#[test]
fn test_chat_request_optional_fields() {
    let request = ChatRequest {
        messages: vec![Message::text(
            "user".to_string(),
            "Test".to_string(),
        )],
        model: None,
        temperature: None,
        max_tokens: None,
        stream: None,
    };

    let json = serde_json::to_string(&request).unwrap();
    // Optional None fields should not be serialized
    assert!(!json.contains("\"model\""));
    assert!(!json.contains("\"temperature\""));
    assert!(!json.contains("\"max_tokens\""));
}
