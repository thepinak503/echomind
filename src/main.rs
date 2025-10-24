use std::fs;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use clap::Parser;
use tokio::io::{self, AsyncReadExt};

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

  some_command | echomind

  echo 'write a function' | echomind --coder --output code.py")]
struct Args {
    #[arg(short = 'c', long)]
    coder: bool,

    #[arg(short = 'o', long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut input = String::new();
    io::stdin().read_to_string(&mut input).await?;

    let messages = if args.coder {
        vec![
            Message {
                role: "system".to_string(),
                content: "You are a code generator. Always and only output raw, runnable code with no explanations, comments, markdown fences, or prose. Do not include code block syntax like triple backticks.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: input.trim().to_string(),
            },
        ]
    } else {
        vec![Message {
            role: "user".to_string(),
            content: input.trim().to_string(),
        }]
    };

    let request = Request {
        messages,
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    let response: Response = client
        .post("https://ch.at/v1/chat/completions")
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    if let Some(choice) = response.choices.first() {
        let content = choice.message.content.trim();
        if let Some(outfile) = &args.output {
            let cleaned = content.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join("\n");
            fs::write(outfile, cleaned)?;
            println!("✅ Code saved to {}", outfile);
        } else {
            println!("{}", content);
        }
    }

    Ok(())
}
