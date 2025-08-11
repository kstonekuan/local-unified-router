use crate::ollama;
use crate::ollama::Message;

pub enum Model {
    Tool,
    Chat,
    Coding,
}

fn create_classification_prompt(prompt: &str) -> Vec<Message> {
    let system_prompt = "You are an expert at categorizing user prompts. Your only task is to classify the user's prompt into one of the following three categories: 'tool', 'chat', or 'coding'. Respond with only a single word: tool, chat, or coding. Do not provide any other text or explanation.".to_string();

    let user_prompt = format!("Categorize the following prompt: \"{}\"", prompt);

    vec![
        Message {
            role: "system".to_string(),
            content: system_prompt,
        },
        Message {
            role: "user".to_string(),
            content: user_prompt,
        },
    ]
}

pub async fn route_with_llm(prompt: &str) -> Model {
    let classification_messages = create_classification_prompt(prompt);

    match ollama::generate("llama3.1:8b", classification_messages).await {
        Ok(response) => {
            let category = response.message.content.trim().to_lowercase();
            if category.contains("tool") {
                Model::Tool
            } else if category.contains("coding") {
                Model::Coding
            } else {
                Model::Chat // Default to chat if the response is not recognized
            }
        }
        Err(e) => {
            eprintln!("Failed to classify prompt: {}", e);
            Model::Chat // Default to chat on an error
        }
    }
}

pub fn get_model_name(model: &Model) -> &str {
    match model {
        Model::Tool => "gpt-oss:20b",
        Model::Chat => "llama3.1:8b",  // Fixed: removed hyphen
        Model::Coding => "gpt-oss:20b", // Replace with qwen when available
    }
}
