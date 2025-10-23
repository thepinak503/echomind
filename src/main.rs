use std::io::{self, Read};
use serde::{Deserialize, Serialize};
use clap::Parser;

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct Request {
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct Response {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Parser)]
#[command(name = "echomind")]
#[command(about = "Send piped input to chat API and print response")]
#[command(long_about = "This tool reads input from stdin (typically piped from another command) and sends it as a message to the chat API at https://ch.at/v1/chat/completions. The response from the API is then printed to stdout.

Example usage:

  echo 'Hello, how are you?' | echomind

  some_command | echomind")]
struct Args;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _args = Args::parse();

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let request = Request {
        messages: vec![Message {
            role: "user".to_string(),
            content: input.trim().to_string(),
        }],
    };

    let client = reqwest::Client::new();
    let response: Response = client
        .post("https://ch.at/v1/chat/completions")
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    if let Some(choice) = response.choices.first() {
        println!("{}", choice.message.content);
    }

    Ok(())
}
