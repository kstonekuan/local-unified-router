mod ollama;
mod router;
mod analytics;

use clap::Parser;
use ollama::Message;
use futures_util::StreamExt;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    prompt: Option<String>,
    
    #[arg(short, long, help = "Run in interactive mode")]
    interactive: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    if args.interactive || args.prompt.is_none() {
        run_interactive_mode().await;
    } else if let Some(prompt) = args.prompt {
        run_single_prompt(prompt).await;
    } else {
        eprintln!("Please provide a prompt with -p or run in interactive mode with -i");
    }
}

async fn run_single_prompt(prompt: String) {
    use std::io::{self, Write};
    let mut conversation_history: Vec<Message> = Vec::new();

    println!("User: {}", prompt);

    print!("\n⚡ Analyzing prompt with llama3.1:8b");
    io::stdout().flush().unwrap();
    
    let model_choice = router::route_with_llm(&prompt).await;
    let model_name = router::get_model_name(&model_choice);
    let category = match model_choice {
        router::Model::Tool => "Tool",
        router::Model::Chat => "Chat",
        router::Model::Coding => "Coding",
    };
    println!("\r✓ Category: {} | Routing to: {}                    ", category, model_name);

    conversation_history.push(Message {
        role: "user".to_string(),
        content: prompt,
    });

    match ollama::generate(model_name, conversation_history.clone()).await {
        Ok(response) => {
            println!("\nAssistant:\n{}", response.message.content);
            analytics::display_analytics(&response);
        }
        Err(e) => eprintln!("Error calling Ollama: {}", e),
    }
}

async fn run_interactive_mode() {
    use std::io::{self, Write};
    
    let mut conversation_history: Vec<Message> = Vec::new();
    
    println!("🤖 Unified LLM Router - Interactive Mode");
    println!("Type 'exit' or 'quit' to end the conversation.\n");

    loop {
        print!("You: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("Goodbye!");
            break;
        }
        
        print!("\n⚡ Analyzing prompt");
        io::stdout().flush().unwrap();
        
        // Show dots animation during routing
        let routing_handle = tokio::spawn(async move {
            let mut dots = 0;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                print!(".");
                io::stdout().flush().unwrap();
                dots += 1;
                if dots > 3 {
                    print!("\r⚡ Analyzing prompt   \r⚡ Analyzing prompt");
                    io::stdout().flush().unwrap();
                    dots = 0;
                }
            }
        });
        
        let model_choice = router::route_with_llm(input).await;
        routing_handle.abort();
        
        let model_name = router::get_model_name(&model_choice);
        let category = match model_choice {
            router::Model::Tool => "Tool",
            router::Model::Chat => "Chat",
            router::Model::Coding => "Coding",
        };
        println!("\r✓ Category: {} | Model: {}                    ", category, model_name);
        
        conversation_history.push(Message {
            role: "user".to_string(),
            content: input.to_string(),
        });
        
        // Stream the response
        print!("\nAssistant: ");
        io::stdout().flush().unwrap();
        
        match stream_response(model_name, conversation_history.clone()).await {
            Ok((content, metrics)) => {
                println!(); // New line after streaming
                
                // Display analytics if we have metrics
                if let Some((total_duration, eval_count, eval_duration)) = metrics {
                    let response = ollama::OllamaResponse {
                        message: ollama::Message {
                            role: "assistant".to_string(),
                            content: content.clone(),
                        },
                        total_duration,
                        eval_count,
                        eval_duration,
                    };
                    analytics::display_analytics(&response);
                }
                
                conversation_history.push(ollama::Message {
                    role: "assistant".to_string(),
                    content,
                });
            }
            Err(e) => eprintln!("\nError calling Ollama: {}", e),
        }
    }
}

async fn stream_response(
    model: &str,
    messages: Vec<Message>,
) -> Result<(String, Option<(u64, usize, u64)>), Box<dyn std::error::Error>> {
    use std::io::{self, Write};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::time::{interval, Duration};
    
    let mut stream = ollama::generate_stream(model, messages).await?;
    let mut full_content = String::new();
    let mut metrics = None;
    
    // Shared flag to stop spinner
    let stop_spinner = Arc::new(AtomicBool::new(false));
    let stop_spinner_clone = stop_spinner.clone();
    
    // Start spinner task
    let spinner_handle = tokio::spawn(async move {
        let spinner_chars = vec!['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let mut interval = interval(Duration::from_millis(80));
        let mut idx = 0;
        
        // Initial spinner display
        print!("{} Thinking...          ", spinner_chars[idx]);
        io::stdout().flush().unwrap();
        
        while !stop_spinner_clone.load(Ordering::Relaxed) {
            tokio::select! {
                _ = interval.tick() => {
                    if !stop_spinner_clone.load(Ordering::Relaxed) {
                        print!("\r{} Thinking...          ", spinner_chars[idx]);
                        io::stdout().flush().unwrap();
                        idx = (idx + 1) % spinner_chars.len();
                    }
                }
            }
        }
        
        // Clear the spinner line
        print!("\r                    \r");
        io::stdout().flush().unwrap();
    });
    
    let mut first_token_received = false;
    
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if !chunk.message.content.is_empty() && !first_token_received {
                    first_token_received = true;
                    // Stop the spinner
                    stop_spinner.store(true, Ordering::Relaxed);
                    // Wait for spinner to clean up
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                
                if !chunk.message.content.is_empty() {
                    print!("{}", chunk.message.content);
                    io::stdout().flush()?;
                    full_content.push_str(&chunk.message.content);
                }
                
                if chunk.done {
                    metrics = Some((
                        chunk.total_duration.unwrap_or(0),
                        chunk.eval_count.unwrap_or(0),
                        chunk.eval_duration.unwrap_or(0),
                    ));
                }
            }
            Err(e) => {
                // Stop spinner on error
                stop_spinner.store(true, Ordering::Relaxed);
                let _ = spinner_handle.await;
                return Err(Box::new(e));
            }
        }
    }
    
    // Ensure spinner is stopped
    stop_spinner.store(true, Ordering::Relaxed);
    
    // Wait for spinner task to complete
    let _ = spinner_handle.await;
    
    Ok((full_content, metrics))
}

