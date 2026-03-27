use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Instant;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRequestPayload {
    pub port: u16,
    pub master_key: String,
    pub model: String,
    pub system_prompt: String,
    pub user_message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRequestResult {
    pub ok: bool,
    pub status: Option<u16>,
    pub duration_ms: u128,
    pub request_preview: String,
    pub response_text: Option<String>,
    pub response_json: Option<String>,
    pub error: Option<String>,
}

pub async fn run_test_request(payload: TestRequestPayload) -> TestRequestResult {
    let body = build_request_body(&payload);
    let request_preview = serde_json::to_string_pretty(&body).unwrap_or_default();

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    if !payload.master_key.trim().is_empty() {
        let token = format!("Bearer {}", payload.master_key);
        if let Ok(value) = HeaderValue::from_str(&token) {
            headers.insert(AUTHORIZATION, value);
        }
    }

    let start = Instant::now();
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://127.0.0.1:{}/chat/completions", payload.port))
        .headers(headers)
        .json(&body)
        .send()
        .await;

    match response {
        Ok(response) => {
            let status = response.status().as_u16();
            let duration_ms = start.elapsed().as_millis();
            match response.text().await {
                Ok(text) => {
                    let parsed = serde_json::from_str::<Value>(&text).ok();
                    let response_text = parsed.as_ref().and_then(extract_response_text);
                    TestRequestResult {
                        ok: (200..300).contains(&status),
                        status: Some(status),
                        duration_ms,
                        request_preview,
                        response_text,
                        response_json: Some(pretty_json(&text)),
                        error: if (200..300).contains(&status) {
                            None
                        } else {
                            Some(format!("本地代理返回非成功状态码: {}", status))
                        },
                    }
                }
                Err(error) => failure_result(request_preview, start.elapsed().as_millis(), error),
            }
        }
        Err(error) => failure_result(request_preview, start.elapsed().as_millis(), error),
    }
}

fn failure_result(
    request_preview: String,
    duration_ms: u128,
    error: impl std::fmt::Display,
) -> TestRequestResult {
    TestRequestResult {
        ok: false,
        status: None,
        duration_ms,
        request_preview,
        response_text: None,
        response_json: None,
        error: Some(format!("测试请求失败: {}", error)),
    }
}

fn pretty_json(raw: &str) -> String {
    serde_json::from_str::<Value>(raw)
        .and_then(|value| serde_json::to_string_pretty(&value))
        .unwrap_or_else(|_| raw.to_string())
}

fn build_request_body(payload: &TestRequestPayload) -> Value {
    let mut messages = Vec::new();
    if !payload.system_prompt.trim().is_empty() {
        messages.push(json!({
            "role": "system",
            "content": payload.system_prompt,
        }));
    }
    messages.push(json!({
        "role": "user",
        "content": payload.user_message,
    }));

    json!({
        "model": payload.model,
        "messages": messages,
        "stream": false
    })
}

fn extract_response_text(value: &Value) -> Option<String> {
    value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

#[cfg(test)]
mod tests {
    use super::{build_request_body, extract_response_text, TestRequestPayload};
    use serde_json::json;

    #[test]
    fn builds_openai_compatible_request_body() {
        let payload = TestRequestPayload {
            port: 4000,
            master_key: "root".to_string(),
            model: "gpt-4o-mini".to_string(),
            system_prompt: "system".to_string(),
            user_message: "hello".to_string(),
        };

        let body = build_request_body(&payload);
        assert_eq!(body["model"], "gpt-4o-mini");
        assert_eq!(body["messages"].as_array().map(Vec::len), Some(2));
    }

    #[test]
    fn extracts_message_content_from_chat_response() {
        let response = json!({
            "choices": [
                {
                    "message": {
                        "content": "LiteLLM 代理已连接。"
                    }
                }
            ]
        });

        assert_eq!(
            extract_response_text(&response).as_deref(),
            Some("LiteLLM 代理已连接。")
        );
    }
}
