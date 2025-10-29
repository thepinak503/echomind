use crate::api::{ApiClient, ChatRequest, Message};
use crate::config::Config;
use crate::error::Result;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

pub struct Repl {
    client: ApiClient,
    config: Config,
    conversation: Vec<Message>,
    temperature: f32,
    max_tokens: Option<u32>,
    model: String,
    stream: bool,
}

impl Repl {
    pub fn new(
        client: ApiClient,
        config: Config,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        model: Option<String>,
        stream: bool,
    ) -> Self {
        Self {
            client,
            conversation: Vec::new(),
            temperature: temperature.unwrap_or(config.defaults.temperature),
            max_tokens: max_tokens.or(config.defaults.max_tokens),
            model: model.unwrap_or(config.api.model.clone()),
            stream,
            config,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("{}", "=== Echomind Interactive Mode ===".cyan().bold());
        println!("Type your message and press Enter. Use {} to exit, {} to clear history.\n",
                 "Ctrl+D or 'exit'".yellow(), "'clear'".yellow());

        let mut rl = DefaultEditor::new()
            .map_err(|e| crate::error::EchomindError::Other(format!("Failed to initialize readline: {}", e)))?;

        loop {
            let readline = rl.readline(&format!("{} ", "You:".green().bold()));

            match readline {
                Ok(line) => {
                    let line = line.trim();

                    if line.is_empty() {
                        continue;
                    }

                    if line == "exit" || line == "quit" {
                        println!("{}", "Goodbye!".cyan());
                        break;
                    }

                    if line == "clear" {
                        self.conversation.clear();
                        println!("{}", "Conversation history cleared.".yellow());
                        continue;
                    }

                    // Add to history
                    let _ = rl.add_history_entry(line);

                    // Add user message to conversation
                    self.conversation.push(Message {
                        role: "user".to_string(),
                        content: line.to_string(),
                    });

                    // Send request
                    let request = ChatRequest {
                        messages: self.conversation.clone(),
                        model: Some(self.model.clone()),
                        temperature: Some(self.temperature),
                        max_tokens: self.max_tokens,
                        stream: if self.stream { Some(true) } else { None },
                    };

                    print!("{} ", "Assistant:".blue().bold());

                    let response = if self.stream {
                        self.client.send_message_stream(request, |chunk| {
                            print!("{}", chunk);
                            use std::io::Write;
                            std::io::stdout().flush().unwrap();
                        }).await?
                    } else {
                        let resp = self.client.send_message(request).await?;
                        println!("{}", resp);
                        resp
                    };

                    if self.stream {
                        println!(); // New line after streaming
                    }

                    // Add assistant response to conversation
                    self.conversation.push(Message {
                        role: "assistant".to_string(),
                        content: response,
                    });

                    println!(); // Empty line for readability
                }
                Err(ReadlineError::Interrupted) => {
                    println!("{}", "^C - Use 'exit' or Ctrl+D to quit".yellow());
                }
                Err(ReadlineError::Eof) => {
                    println!("{}", "Goodbye!".cyan());
                    break;
                }
                Err(err) => {
                    return Err(crate::error::EchomindError::Other(format!("Readline error: {}", err)));
                }
            }
        }

        Ok(())
    }
}
