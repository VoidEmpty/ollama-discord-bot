use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub model_name: String,
    pub mirostat: u8,
    pub context_size: u32,
    pub bot_token: String,
    pub system_prompt: String,
}
