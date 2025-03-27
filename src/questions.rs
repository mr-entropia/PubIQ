pub mod loader {
    use super::structure::Root;

    pub fn load_questions_from_file(path: &str) -> Option<Root> {
        let in_file = std::fs::read_to_string(path);

        let data = match in_file {
            Ok(s) => s,
            Err(_e) => {
                eprintln!("Questions file ({}) was not found.", path);
                return None;
            }
        };

        match serde_json::from_str(&data) {
            Ok(val) => {
                return Some(val);
            }
            Err(e) => {
                eprintln!(
                    "Questions file ({}) is invalid.\n\nError: {}",
                    path,
                    e.to_string()
                );
                return None;
            }
        };
    }
}

pub mod structure {
    use serde::{Deserialize, Serialize};

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct Root {
        pub metadata: Metadata,
        pub questions: Vec<Question>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct Metadata {
        pub author: String,
        pub time: i64,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct Question {
        pub id: i64,
        pub category: Vec<String>,
        pub question: String,
        #[serde(rename = "question_tts")]
        pub question_tts: String,
        #[serde(rename = "context_information")]
        pub context_information: String,
        #[serde(rename = "context_information_tts")]
        pub context_information_tts: String,
        pub correct: String,
        #[serde(rename = "correct_tts")]
        pub correct_tts: String,
        #[serde(rename = "incorrect_1")]
        pub incorrect_1: String,
        #[serde(rename = "incorrect_2")]
        pub incorrect_2: String,
        #[serde(rename = "incorrect_3")]
        pub incorrect_3: String,
        pub answer_options: Option<Vec<String>>,
    }
}
