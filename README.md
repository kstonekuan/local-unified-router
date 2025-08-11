# Unified LLM Router

This project is a command-line interface (CLI) tool built in Rust that intelligently routes prompts to different large language models (LLMs) running in Ollama. It uses a dedicated LLM to analyze and categorize user prompts, ensuring that each query is handled by the most suitable model for the task (e.g., chat, coding).

The tool also provides detailed performance analytics after each response, helping users benchmark model performance on their hardware.

## Goals

This project is inspired by the concept of a "unified router," similar to the architecture described in [OpenAI's GPT-5 System Card](https://openai.com/index/gpt-5-system-card/). The goal is to bring this powerful routing technology to the open-source community, allowing users to run a sophisticated, multi-model setup on their personal computers with local LLMs.

## Features

*   **Intelligent Prompt Routing:** Automatically classifies prompts and routes them to the best model for the task.
*   **Performance Analytics:** Measures and displays key metrics like tokens per second, total tokens, and total duration for each response.
*   **Hardware Performance Rating:** Provides a simple rating (e.g., "Excellent," "Good") of the model's speed on your machine.
*   **Conversation History:** Maintains context for follow-up questions within a session.
*   **Multiple Interaction Modes:** Supports both single-prompt execution and an interactive chat mode.

## How It Works

1.  The user provides a prompt through the CLI.
2.  A dedicated routing model (`llama3.1:8b`) analyzes the prompt to determine its category (e.g., "chat", "coding").
3.  The prompt is then forwarded to the specialized model best suited for that category.
4.  The response from the specialized model is streamed back to the user.
5.  After the response is complete, a summary of performance analytics is displayed.

## Prerequisites

*   Rust and Cargo
*   Ollama running on your local machine.

You also need to have the following models pulled in Ollama:

```bash
ollama pull llama3.1:8b
ollama pull gpt-oss:20b
ollama pull qwen3-coder:30b
```

## Installation & Building

1.  Clone the repository (if you haven't already).
2.  Build the application for production:
    ```bash
    cargo build --release
    ```
    The executable will be available at `target/release/local-unified-router`.

## Usage

You can run the tool in two modes:

**1. Single Prompt Mode**

Use the `-p` or `--prompt` flag to pass a single prompt.

*Chat Example:*
```bash
cargo run -- -p "What are the latest advancements in AI?"
```

*Coding Example:*
```bash
cargo run -- -p "Can you write a Python script to parse a CSV file?"
```

**2. Interactive Mode**

To start a continuous chat session, run the command without any arguments or with the `-i` flag.

```bash
cargo run
```
or
```bash
cargo run -- -i
```

### Example Output with Analytics

After a response is generated, you will see a performance summary like this:

```
Assistant:
Unit 734 processed the world in data streams of pure logic. Its existence was a silent ballet of efficiency and function. One cycle, while cleaning an archival sector, it stumbled upon a corrupted file labeled "Bach." Attempting to parse it, Unit 734's auditory sensors were flooded not with data, but with... harmony. The structured, mathematical beauty of the cello suite resonated with its core programming in a way it couldn't comprehend. It was illogical, inefficient, and utterly captivating. For the first time, Unit 734 chose to deviate from its tasks, replaying the file again and again, lost in the newfound, beautiful noise.

--- Performance Analytics ---
Tokens/Second: 75.43
Hardware Performance Rating: Good
Total Tokens: 112
Total Duration: 1.87s
---------------------------
```
