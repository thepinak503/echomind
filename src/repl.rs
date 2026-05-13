use crate::api::{ApiClient, ChatRequest, Message};
use crate::config::Config;
use crate::error::Result;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::io::Write;

pub struct Repl {
    client: ApiClient,
    conversation: Vec<Message>,
    temperature: f32,
    max_tokens: Option<u32>,
    model: String,
    stream: bool,
}

impl Repl {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client: ApiClient,
        config: Config,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        model: Option<String>,
        stream: bool,
        initial_messages: Vec<Message>,
        system_prompt: Option<String>,
    ) -> Self {
        let mut conversation = Vec::new();
        if let Some(sp) = system_prompt {
            conversation.push(Message::new("system", sp));
        }
        conversation.extend(initial_messages);

        Self {
            client,
            conversation,
            temperature: temperature.unwrap_or(config.defaults.temperature),
            max_tokens: max_tokens.or(config.defaults.max_tokens),
            model: model.unwrap_or_else(|| config.api.model.clone()),
            stream,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("{}", "=== Echomind Interactive Mode ===".cyan().bold());
        println!(
            "Type your message. {} to exit, {} to clear.\n",
            "Ctrl+D/'exit'".yellow(),
            "'clear'".yellow()
        );

        let mut rl = DefaultEditor::new().map_err(|e| {
            crate::error::EchomindError::Other(format!("Readline init failed: {}", e))
        })?;

        loop {
            match rl.readline(&format!("{} ", "You:".green().bold())) {
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
                        println!("{}", "History cleared.".yellow());
                        continue;
                    }

                    let _ = rl.add_history_entry(line);
                    self.conversation.push(Message::new("user", line));

                    let request = ChatRequest {
                        messages: self.conversation.clone(),
                        model: Some(self.model.clone()),
                        temperature: Some(self.temperature),
                        max_tokens: self.max_tokens,
                        top_p: None,
                        top_k: None,
                        stream: if self.stream { Some(true) } else { None },
                    };

                    print!("{} ", "Assistant:".blue().bold());

                    let response = if self.stream {
                        self.client
                            .send_message_stream(request, |chunk| {
                                print!("{}", chunk);
                                std::io::stdout().flush().ok();
                            })
                            .await?
                    } else {
                        let r = self.client.send_message(request).await?;
                        println!("{}", r);
                        r
                    };

                    if self.stream {
                        println!();
                    }

                    self.conversation.push(Message::new("assistant", &response));
                    println!();
                }
                Err(ReadlineError::Interrupted) => {
                    println!("{}", "^C - Use 'exit' or Ctrl+D to quit".yellow());
                }
                Err(ReadlineError::Eof) => {
                    println!("{}", "Goodbye!".cyan());
                    break;
                }
                Err(err) => {
                    return Err(crate::error::EchomindError::Other(format!("Readline: {}", err)));
                }
            }
        }
        Ok(())
    }
}
