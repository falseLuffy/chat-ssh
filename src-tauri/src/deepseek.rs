use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

pub async fn generate_command(prompt: &str, api_key: &str) -> Result<String, String> {
    let client = Client::new();
    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&ChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "你是一个专业的 Shell 指令专家。你只需返回对应的 Shell 指令，不要任何解释。".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: format!("帮我生成对应的 Shell 指令：{}", prompt),
                },
            ],
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let result: ChatResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(result.choices[0].message.content.trim().to_string())
}

pub async fn analyze_risk(command: &str, api_key: &str) -> Result<String, String> {
    let client = Client::new();
    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&ChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "你是一个 Linux 安全审计专家。请分析以下指令的潜在风险，并给出评估意见（风险等级、潜在后果、改进建议）。".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: format!("请审查此指令的风险：{}", command),
                },
            ],
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let result: ChatResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(result.choices[0].message.content.trim().to_string())
}
