pub mod google {
    use serde_derive::Deserialize;
    use serde_derive::Serialize;
    use serde_json::json;

    const GOOGLE_API_ENDPOINT: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=";

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct Root {
        pub candidates: Vec<Candidate>,
        #[serde(rename = "usageMetadata")]
        pub usage_metadata: UsageMetadata,
        #[serde(rename = "modelVersion")]
        pub model_version: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct Candidate {
        pub content: Content,
        #[serde(rename = "finishReason")]
        pub finish_reason: String,
        #[serde(rename = "avgLogprobs")]
        pub avg_logprobs: f64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct Content {
        pub parts: Vec<Part>,
        pub role: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct Part {
        pub text: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct UsageMetadata {
        #[serde(rename = "promptTokenCount")]
        pub prompt_token_count: i64,
        #[serde(rename = "candidatesTokenCount")]
        pub candidates_token_count: i64,
        #[serde(rename = "totalTokenCount")]
        pub total_token_count: i64,
        #[serde(rename = "promptTokensDetails")]
        pub prompt_tokens_details: Vec<PromptTokensDetail>,
        #[serde(rename = "candidatesTokensDetails")]
        pub candidates_tokens_details: Vec<CandidatesTokensDetail>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct PromptTokensDetail {
        pub modality: String,
        #[serde(rename = "tokenCount")]
        pub token_count: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct CandidatesTokensDetail {
        pub modality: String,
        #[serde(rename = "tokenCount")]
        pub token_count: i64,
    }

    pub fn prompt_gemini(prompt: String) -> Result<String, String> {
        let api_key = match std::env::var("GOOGLE_GENAI_STUDIO_API_KEY") {
            Ok(api_key) => api_key,
            Err(_) => {
                panic!("Unable to get Google GenAI Studio API key. Use an environment variable called GOOGLE_GENAI_STUDIO_API_KEY to set it.");
            }
        };

        let body = json!({
            "contents": [
                {
                    "role": "user",
                    "parts": [
                        {
                            "text": prompt
                        }
                    ]
                }
            ],
            "generationConfig": {
                "temperature": 1,
                "topK": 40,
                "topP": 0.95,
                "maxOutputTokens": 1024,
                "responseMimeType": "text/plain"
            }
        })
        .to_string();

        let mut response =
            match ureq::post(format!("{}{}", GOOGLE_API_ENDPOINT, api_key)).send(body) {
                Ok(response) => response,
                Err(error) => {
                    return Err(format!("{}", error));
                }
            };

        let response_string = match response.body_mut().read_to_string() {
            Ok(response_string) => response_string,
            Err(error) => {
                return Err(format!("{}", error));
            }
        };

        let response_json: Root = match serde_json::from_str(&response_string) {
            Ok(json) => json,
            Err(error) => {
                return Err(format!("{}", error));
            }
        };

        return Ok(response_json.candidates[0].content.parts[0]
            .text
            .as_str()
            .trim_end()
            .to_string());
    }
}

pub mod elevenlabs {
    use crate::helpers::time_helpers::uptime_ms;
    use serde_json::json;
    use std::{fs::File, io::Write, path::Path};

    const ELEVENLABS_API_ENDPOINT: &str = "https://api.elevenlabs.io/v1/text-to-speech/";
    const ELEVENLABS_VOICE_ID: &str = "YSabzCJMvEHDduIDMdwV"; // Aurora
    const ELEVENLABS_MODEL_ID: &str = "eleven_flash_v2_5";

    pub enum AudioType {
        Question,
        Answer,
        NoCache,
    }

    fn check_if_audio_exists(question_id: &i64, audio_type: &AudioType) -> Option<String> {
        //let mut filename = "".to_string();
        let filename = match audio_type {
            AudioType::Question => format!("audio/q-{}.mp3", question_id.to_string()),
            AudioType::Answer => format!("audio/a-{}.mp3", question_id.to_string()),
            AudioType::NoCache => {
                return None;
            }
        };
        let final_filename = format!("web/{}", &filename);
        if Path::new(&final_filename).exists() {
            return Some(filename);
        } else {
            return None;
        }
    }

    pub fn generate_speech(
        text: &String,
        question_id: &i64,
        audio_type: AudioType,
    ) -> Result<String, String> {
        //generate_speech_dummy(text, question_id, audio_type)
        generate_speech_elevenlabs(text, question_id, audio_type)
    }

    fn generate_speech_dummy(
        text: &String,
        question_id: &i64,
        audio_type: AudioType,
    ) -> Result<String, String> {
        println!("Generate speech (dummy): {}", text);
        match check_if_audio_exists(question_id, &audio_type) {
            Some(filename) => {
                println!("Speech is cached, returning {}", filename);
                return Ok(filename);
            }
            None => {
                println!("Not cached");
            }
        };
        let filename = format!("audio/{}.mp3", uptime_ms().to_string());
        let source_file = match &audio_type {
            AudioType::Answer => "blip.mp3".to_string(),
            AudioType::Question => "blip.mp3".to_string(),
            AudioType::NoCache => "blip.mp3".to_string(),
        };
        match std::fs::copy(
            format!("web/audio/{}", source_file),
            format!("web/{}", &filename),
        ) {
            Ok(_) => {
                println!("Filename: {}", filename);
                return Ok(filename);
            }
            Err(error) => {
                println!("Failed: {}", error);
                return Err("".to_string());
            }
        }
    }

    fn generate_speech_elevenlabs(
        text: &String,
        question_id: &i64,
        audio_type: AudioType,
    ) -> Result<String, String> {
        let api_key = match std::env::var("ELEVENLABS_API_KEY") {
            Ok(api_key) => api_key,
            Err(_) => {
                panic!("Unable to get Elevenlabs API key. Use an environment variable called ELEVENLABS_API_KEY to set it. If you don't have an API key, use generate_speech_dummy function instead.");
            }
        };

        println!("Generate speech: {}", text);
        match check_if_audio_exists(&question_id, &audio_type) {
            Some(filename) => {
                println!("Speech is cached, returning {}", filename);
                return Ok(filename);
            }
            None => {}
        };
        let body = json!({
            "text": text,
            "model_id": ELEVENLABS_MODEL_ID,
            "voice_settings": {
                "stability": 0.51,
                "similarity_boost": 0.75,
                "speed": 0.89
            }
        })
        .to_string();

        let mut response = match ureq::post(format!(
            "{}{}",
            ELEVENLABS_API_ENDPOINT, ELEVENLABS_VOICE_ID
        ))
        .header("xi-api-key", api_key)
        .header("Content-Type", "application/json")
        .send(body)
        {
            Ok(response) => response,
            Err(error) => {
                return Err(format!("{}", error));
            }
        };

        let body = match response.body_mut().read_to_vec() {
            Ok(vec) => vec,
            Err(error) => {
                return Err(format!("{}", error));
            }
        };

        let filename: String = match audio_type {
            AudioType::Answer => format!("audio/a-{}.mp3", question_id).to_string(),
            AudioType::Question => format!("audio/q-{}.mp3", question_id).to_string(),
            AudioType::NoCache => format!("audio/nocache-{}.mp3", uptime_ms()).to_string(),
        };

        let mut file = match File::create(format!("web/{}", &filename)) {
            Ok(file) => file,
            Err(error) => {
                return Err(format!("{}", error));
            }
        };
        match file.write_all(&body) {
            Ok(_) => {
                return Ok(filename);
            }
            Err(error) => {
                return Err(format!("{}", error));
            }
        }
    }
}
