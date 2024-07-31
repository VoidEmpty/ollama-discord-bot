use ollama_rs::{
    generation::{
        completion::{request::GenerationRequest, GenerationContext},
        options::GenerationOptions,
    },
    Ollama,
};
use std::collections::HashMap;

pub struct Model {
    ollama: Ollama,
    model_name: String,
    generation_options: GenerationOptions,
    system_prompt: String,
    context: HashMap<String, GenerationContext>,
}

impl Model {
    pub fn new(
        model_name: impl Into<String>,
        generation_options: GenerationOptions,
        system_prompt: impl Into<String>,
    ) -> Self {
        Self {
            ollama: Ollama::default(),
            model_name: model_name.into(),
            generation_options,
            system_prompt: system_prompt.into(),
            context: HashMap::new(),
        }
    }

    fn get_context(&self, id: impl AsRef<str>) -> Option<&GenerationContext> {
        self.context.get(id.as_ref())
    }

    fn set_context(&mut self, id: impl Into<String>, context: GenerationContext) {
        self.context.insert(id.into(), context);
    }

    pub async fn send_message(
        &mut self,
        user_name: impl AsRef<str>,
        prompt: impl AsRef<str>,
    ) -> Option<String> {
        let user_name = user_name.as_ref();
        let prompt = prompt.as_ref();
        let message = format!("[{user_name}]: {prompt}");

        let res = self
            .ollama
            .generate(
                GenerationRequest::new(self.model_name.clone(), message)
                    .options(self.generation_options.clone())
                    .system(self.system_prompt.clone())
                    .context(
                        self.get_context(user_name)
                            .unwrap_or(&GenerationContext(Vec::new()))
                            .to_owned(),
                    ),
            )
            .await;

        if let Ok(response) = res {
            if let Some(context) = response.context {
                self.set_context(user_name, context);
            }
            Some(response.response)
        } else {
            Some(String::default())
        }
    }
}
