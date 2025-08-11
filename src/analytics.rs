use crate::ollama::OllamaResponse;

// Calculates tokens per second from the response data
fn calculate_tokens_per_second(response: &OllamaResponse) -> f64 {
    if response.eval_duration > 0 {
        let eval_duration_secs = response.eval_duration as f64 / 1_000_000_000.0;
        response.eval_count as f64 / eval_duration_secs
    } else {
        0.0
    }
}

// Rates the performance based on tokens per second
// These thresholds are examples and should be adjusted based on your hardware.
fn rate_performance(tokens_per_second: f64) -> String {
    if tokens_per_second > 100.0 {
        "Excellent".to_string()
    } else if tokens_per_second > 50.0 {
        "Good".to_string()
    } else if tokens_per_second > 20.0 {
        "Moderate".to_string()
    } else {
        "Slow".to_string()
    }
}

// Main function to generate and display the analytics
pub fn display_analytics(response: &OllamaResponse) {
    let tokens_per_second = calculate_tokens_per_second(response);
    let rating = rate_performance(tokens_per_second);

    println!("\n--- Performance Analytics ---");
    println!("Tokens/Second: {:.2}", tokens_per_second);
    println!("Hardware Performance Rating: {}", rating);
    println!("Total Tokens: {}", response.eval_count);
    println!("Total Duration: {:.2}s", response.total_duration as f64 / 1_000_000_000.0);
    println!("---------------------------\n");
}