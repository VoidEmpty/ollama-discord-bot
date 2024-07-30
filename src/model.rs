use ollama_rs::{
    generation::{
        completion::{request::GenerationRequest, GenerationContext},
        options::GenerationOptions,
    },
    Ollama,
};

pub struct Model {
    ollama: Ollama,
    model_name: String,
    generation_options: GenerationOptions,
    system_prompt: String,
    context: GenerationContext,
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
            context: GenerationContext(vec![0]),
        }
    }

    pub async fn send_message(
        &mut self,
        user_name: impl Into<String> + std::fmt::Display,
        prompt: impl Into<String> + std::fmt::Display,
    ) -> Option<String> {
        let prompt = prompt.into();
        let message = format!("[{user_name}]: {prompt}");

        let res = self
            .ollama
            .generate(
                GenerationRequest::new(self.model_name.clone(), message)
                    .options(self.generation_options.clone())
                    .system(self.system_prompt.clone())
                    .context(self.context.clone()),
            )
            .await;

        if let Ok(response) = res {
            if let Some(context) = response.context {
                self.context = context;
            }
            Some(response.response)
        } else {
            Some(String::default())
        }
    }
}
