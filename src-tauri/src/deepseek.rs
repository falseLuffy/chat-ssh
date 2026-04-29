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

#[derive(Serialize, Deserialize)]
pub struct ScriptAnalysisResult {
    pub description: String,
    pub has_password_leak: bool,
}

pub async fn analyze_script_with_ai(script: &str, api_key: &str) -> Result<ScriptAnalysisResult, String> {
    let client = Client::new();
    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&ChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "你是一个专业的脚本分析助手。你需要分析用户提供的脚本，以 JSON 格式返回分析结果。JSON 必须包含两个字段：1. 'description' (字符串，简短总结该脚本的用途、功能和执行后的潜在后果)，2. 'has_password_leak' (布尔值，如果脚本中包含明文密码、密钥、敏感 Token 等高危信息，则为 true，否则为 false)。除了合法的 JSON，不要输出任何其他内容。".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: format!("请分析此脚本：\n{}", script),
                },
            ],
        })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let result: ChatResponse = response.json().await.map_err(|e| e.to_string())?;
    let content = result.choices[0].message.content.trim();
    
    // Attempt to parse JSON. Some models might wrap it in ```json ... ```
    let clean_content = if content.starts_with("```json") && content.ends_with("```") {
        content[7..content.len()-3].trim()
    } else if content.starts_with("```") && content.ends_with("```") {
        content[3..content.len()-3].trim()
    } else {
        content
    };

    serde_json::from_str::<ScriptAnalysisResult>(clean_content)
        .map_err(|e| format!("Failed to parse AI response: {}", e))
}
