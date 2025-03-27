mod external_apis;
mod game;
mod helpers;
mod questions;
/// PubIQ
/// https://github.com/mr-entropia
/// Licensed under AGPL-3.0
mod rest_api;

use game::{
    controller::run_game_controller,
    state::{Answers, GameState, Questions},
};
use questions::{loader::load_questions_from_file, structure::Question};
use rest_api::rest_http::run_rest_http_api;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
    vec,
};

fn main() {
    let all_questions: &'static questions::structure::Root =
        match load_questions_from_file("questions.json") {
            Some(questions) => Box::leak(Box::new(questions)),
            None => {
                eprintln!("Unable to load questions. Exiting.");
                std::process::exit(-1);
            }
        };

    let game_state = Arc::new(Mutex::new(GameState {
        game_stage: game::state::GameStage::WaitingForPlayers,
        question: Question {
            id: 0,
            category: vec![],
            question: "".to_string(),
            question_tts: "".to_string(),
            context_information: "".to_string(),
            context_information_tts: "".to_string(),
            correct: "".to_string(),
            correct_tts: "".to_string(),
            incorrect_1: "".to_string(),
            incorrect_2: "".to_string(),
            incorrect_3: "".to_string(),
            answer_options: None,
        },
        question_number: 0,
        question_stage: game::state::QuestionStage::QuestionIntroduction,
        question_start_time: 0,
        question_limit: 5,
        players: vec![],
        proceed_flag: false,
        newgame_flag: false,
        audio: None,
        tts_text: None,
        scores: vec![],
    }));

    let empty_questions: Vec<Questions> = vec![];
    let empty_answers: Vec<Answers> = vec![];

    let questions = Arc::new(Mutex::new(empty_questions));
    let answers = Arc::new(Mutex::new(empty_answers));

    let game_state_clone = game_state.clone();
    let questions_clone = questions.clone();
    let answers_clone = answers.clone();

    // Start REST API
    let builder = thread::Builder::new().name("REST API".into());
    match builder.spawn(move || {
        run_rest_http_api(game_state_clone, questions_clone, answers_clone);
    }) {
        Ok(_) => (),
        Err(_) => {
            std::process::exit(-1);
        }
    }

    let game_state_clone = game_state.clone();
    let questions_clone = questions.clone();
    let answers_clone = answers.clone();

    // Start game controller
    let builder = thread::Builder::new().name("Game controller".into());
    match builder.spawn(move || {
        run_game_controller(
            game_state_clone,
            questions_clone,
            answers_clone,
            all_questions,
        );
    }) {
        Ok(_) => (),
        Err(_) => {
            std::process::exit(-1);
        }
    }

    loop {
        thread::sleep(Duration::from_millis(100));
    }
}
