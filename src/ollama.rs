use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct OllamaRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
}

// Updated to capture detailed performance metrics from Ollama
#[derive(Deserialize, Debug)]
pub struct OllamaResponse {
    pub message: Message,
    pub total_duration: u64, // Total time for the request in nanoseconds
    pub eval_count: usize,   // Number of tokens in the response
    pub eval_duration: u64,  // Time to evaluate the response in nanoseconds
}


// The generate function now returns the full OllamaResponse
pub async fn generate(
    model: &str,
    messages: Vec<Message>,
) -> Result<OllamaResponse, reqwest::Error> {
    let client = Client::new();
    let request = OllamaRequest {
        model: model.to_string(),
        messages,
        stream: false,
    };

    let response: OllamaResponse = client
        .post("http://localhost:11434/api/chat")
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    Ok(response)
}

// Streaming response chunk
#[derive(Deserialize, Debug)]
pub struct StreamChunk {
    pub message: Message,
    pub done: bool,
    pub total_duration: Option<u64>,
    pub eval_count: Option<usize>,
    pub eval_duration: Option<u64>,
}

// Streaming version of generate
pub async fn generate_stream(
    model: &str,
    messages: Vec<Message>,
) -> Result<impl futures_util::Stream<Item = Result<StreamChunk, reqwest::Error>>, reqwest::Error> {
    use futures_util::StreamExt;
    
    let client = Client::new();
    let request = OllamaRequest {
        model: model.to_string(),
        messages,
        stream: true,
    };

    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&request)
        .send()
        .await?;

    let stream = response.bytes_stream().map(|chunk| {
        match chunk {
            Ok(bytes) => {
                let text = String::from_utf8_lossy(&bytes);
                
                // Process each line separately (streaming responses are newline-delimited JSON)
                for line in text.trim().split('\n') {
                    if !line.is_empty() {
                        match serde_json::from_str::<StreamChunk>(line) {
                            Ok(chunk) => return Ok(chunk),
                            Err(_) => continue, // Skip invalid lines
                        }
                    }
                }
                
                // Return empty chunk if no valid JSON found
                Ok(StreamChunk {
                    message: Message {
                        role: "assistant".to_string(),
                        content: String::new(),
                    },
                    done: false,
                    total_duration: None,
                    eval_count: None,
                    eval_duration: None,
                })
            }
            Err(e) => Err(e),
        }
    });

    Ok(stream)
}

