pub mod state {
    use super::controller::Points;
    use crate::questions::structure::Question;
    use core::fmt;
    use uuid::Uuid;

    #[derive(Clone, PartialEq)]
    pub struct Player {
        pub name: String,
        pub uuid: Uuid,
        pub last_seen: u64,
        pub score: i32,
    }

    #[derive(PartialEq)]
    pub enum QuestionStage {
        QuestionIntroduction,
        QuestionAnswerTime,
        QuestionFinished,
    }

    impl fmt::Display for QuestionStage {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let display = match *self {
                QuestionStage::QuestionIntroduction => "QuestionIntroduction".to_string(),
                QuestionStage::QuestionAnswerTime => "QuestionAnswerTime".to_string(),
                QuestionStage::QuestionFinished => "QuestionFinished".to_string(),
            };
            write!(f, "{}", display)
        }
    }

    #[derive(PartialEq)]
    pub enum GameStage {
        WaitingForPlayers,
        IntroducePlayers,
        GameInProgress,
        ResultsShow,
        //GameFinished,
    }

    impl fmt::Display for GameStage {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
            let display = match *self {
                GameStage::WaitingForPlayers => "WaitingForPlayers".to_string(),
                GameStage::GameInProgress => "GameInProgress".to_string(),
                GameStage::ResultsShow => "ResultsShow".to_string(),
                GameStage::IntroducePlayers => "IntroducePlayers".to_string(),
                //GameStage::GameFinished => "GameFinished".to_string(),
            };
            write!(f, "{}", display)
        }
    }

    #[derive(PartialEq)]
    pub struct GameState {
        pub game_stage: GameStage,
        pub question: Question,
        pub question_number: u64,
        pub question_stage: QuestionStage,
        pub question_start_time: u64,
        pub question_limit: u64,
        pub players: Vec<Player>,
        pub proceed_flag: bool,
        pub newgame_flag: bool,
        pub audio: Option<String>,
        pub tts_text: Option<String>,
        pub scores: Vec<Points>,
    }

    #[derive(Debug, Clone)]
    pub struct Questions {
        pub question_number: u64,
        pub question_id: i64,
    }

    #[derive(Debug, Clone)]
    pub struct Answers {
        pub question_number: u64,
        pub answer: String,
        pub player_uuid: Uuid,
    }
}

pub mod controller {
    use super::state::{Answers, GameStage, GameState, QuestionStage, Questions};
    use crate::{
        external_apis::{
            elevenlabs::{generate_speech, AudioType},
            google::prompt_gemini,
        },
        helpers::{
            natural_language::{
                correct_answer_and_context_announcement, get_player_names_for_tts,
                prompt_for_player_introduction, prompt_for_winner_announcement,
            },
            time_helpers::uptime_ms,
        },
        questions::structure::{Question, Root},
    };
    use rand::{seq::SliceRandom, Rng};
    use std::{
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    #[derive(Clone, Debug, PartialEq, serde::Serialize)]
    pub struct Points {
        pub player_name: String,
        pub points: u32,
    }

    pub fn run_game_controller(
        game_state: Arc<Mutex<GameState>>,
        questions: Arc<Mutex<Vec<Questions>>>,
        answers: Arc<Mutex<Vec<Answers>>>,
        all_questions: &Root,
    ) {
        println!("Game controller started");
        loop {
            {
                let mut game_state_mutex = match game_state.lock() {
                    Ok(mutex) => mutex,
                    Err(poisoned_mutex) => poisoned_mutex.into_inner(),
                };

                let answers_mutex = match answers.lock() {
                    Ok(mutex) => mutex,
                    Err(poisoned_mutex) => poisoned_mutex.into_inner(),
                };

                match game_state_mutex.game_stage {
                    GameStage::WaitingForPlayers => {
                        if game_state_mutex.proceed_flag {
                            game_state_mutex.game_stage = GameStage::IntroducePlayers;
                            game_state_mutex.proceed_flag = false;
                            println!(
                                "Proceed triggered -- state {}",
                                game_state_mutex.game_stage.to_string()
                            );
                            game_state_mutex.question_number = 1;
                            game_state_mutex.question = get_new_question(
                                all_questions,
                                questions.clone(),
                                game_state_mutex.question_number,
                            );
                            game_state_mutex.question_stage = QuestionStage::QuestionIntroduction;
                            game_state_mutex.question.answer_options =
                                Some(shuffle_answers(&game_state_mutex.question));

                            let mut list_of_players: Vec<String> = vec![];
                            for player in game_state_mutex.players.iter() {
                                list_of_players.push(player.name.clone());
                            }
                            match prompt_gemini(prompt_for_player_introduction(
                                get_player_names_for_tts(list_of_players),
                            )) {
                                Ok(tts_text) => {
                                    match generate_speech(&tts_text, &0, AudioType::NoCache) {
                                        Ok(audio_filename) => {
                                            game_state_mutex.tts_text = Some(tts_text);
                                            game_state_mutex.audio = Some(audio_filename);
                                        }
                                        Err(_) => {
                                            game_state_mutex.tts_text = None;
                                            game_state_mutex.audio = None;
                                        }
                                    };
                                }
                                Err(_) => {
                                    game_state_mutex.tts_text = None;
                                    game_state_mutex.audio = None;
                                }
                            };
                        }
                    }
                    GameStage::IntroducePlayers => {
                        if game_state_mutex.proceed_flag {
                            println!(
                                "Proceed triggered -- state {}",
                                game_state_mutex.game_stage.to_string()
                            );
                            game_state_mutex.proceed_flag = false;
                            match generate_speech(
                                &game_state_mutex.question.question_tts,
                                &game_state_mutex.question.id,
                                AudioType::Question,
                            ) {
                                Ok(audio_filename) => {
                                    game_state_mutex.audio = Some(audio_filename);
                                }
                                Err(_) => {
                                    game_state_mutex.audio = None;
                                }
                            };
                            game_state_mutex.game_stage = GameStage::GameInProgress;
                            game_state_mutex.question_start_time = uptime_ms();
                        }
                    }
                    GameStage::GameInProgress => match game_state_mutex.question_stage {
                        QuestionStage::QuestionIntroduction => {
                            if game_state_mutex.proceed_flag
                                || uptime_ms() > game_state_mutex.question_start_time + 30000
                            {
                                println!(
                                    "Proceed triggered -- state {}",
                                    game_state_mutex.game_stage.to_string()
                                );
                                game_state_mutex.proceed_flag = false;
                                game_state_mutex.question_stage = QuestionStage::QuestionAnswerTime;
                            }
                        }
                        QuestionStage::QuestionAnswerTime => {
                            game_state_mutex.proceed_flag = false;
                            if count_players_answered_to_question(
                                &answers_mutex,
                                game_state_mutex.question_number,
                            ) == game_state_mutex.players.len() as u64
                                || uptime_ms() > game_state_mutex.question_start_time + 60000
                            {
                                println!(
                                    "Proceed triggered -- state {}",
                                    game_state_mutex.game_stage.to_string()
                                );
                                game_state_mutex.question_stage = QuestionStage::QuestionFinished;
                                match generate_speech(
                                    &correct_answer_and_context_announcement(
                                        &game_state_mutex.question.correct_tts,
                                        &game_state_mutex.question.context_information_tts,
                                    ),
                                    &game_state_mutex.question.id,
                                    AudioType::Answer,
                                ) {
                                    Ok(audio_filename) => {
                                        game_state_mutex.audio = Some(audio_filename);
                                    }
                                    Err(_) => {
                                        game_state_mutex.audio = None;
                                    }
                                };
                            }
                        }
                        QuestionStage::QuestionFinished => {
                            if game_state_mutex.proceed_flag
                                || uptime_ms() > game_state_mutex.question_start_time + 45000
                            {
                                game_state_mutex.proceed_flag = false;
                                game_state_mutex.question_number += 1;
                                if game_state_mutex.question_number
                                    > game_state_mutex.question_limit
                                {
                                    println!("Game finished!");
                                    game_state_mutex.game_stage = GameStage::ResultsShow;
                                    let _scores = match count_points(
                                        all_questions,
                                        &game_state_mutex,
                                        questions.clone(),
                                        &answers_mutex,
                                    ) {
                                        Some(scores) => {
                                            let winner_name = &scores[0].player_name;
                                            let winner_points = &scores[0].points;
                                            match prompt_gemini(prompt_for_winner_announcement(
                                                winner_name.to_string(),
                                                winner_points.to_string(),
                                            )) {
                                                Ok(tts_text) => {
                                                    match generate_speech(
                                                        &tts_text,
                                                        &0,
                                                        AudioType::NoCache,
                                                    ) {
                                                        Ok(audio_filename) => {
                                                            game_state_mutex.tts_text =
                                                                Some(tts_text);
                                                            game_state_mutex.audio =
                                                                Some(audio_filename);
                                                        }
                                                        Err(_) => {
                                                            game_state_mutex.tts_text = None;
                                                            game_state_mutex.audio = None;
                                                        }
                                                    };
                                                }
                                                Err(_) => {
                                                    game_state_mutex.tts_text = None;
                                                    game_state_mutex.audio = None;
                                                }
                                            };
                                            game_state_mutex.scores = scores.clone();
                                            scores
                                        }
                                        None => vec![],
                                    };
                                } else {
                                    println!("\nNew question\n");
                                    game_state_mutex.question = get_new_question(
                                        all_questions,
                                        questions.clone(),
                                        game_state_mutex.question_number,
                                    );
                                    match generate_speech(
                                        &game_state_mutex.question.question_tts,
                                        &game_state_mutex.question.id,
                                        AudioType::Question,
                                    ) {
                                        Ok(audio_filename) => {
                                            game_state_mutex.audio = Some(audio_filename);
                                        }
                                        Err(_) => {
                                            game_state_mutex.audio = None;
                                        }
                                    };
                                    game_state_mutex.question_stage =
                                        QuestionStage::QuestionIntroduction;
                                    game_state_mutex.question_start_time = uptime_ms();
                                    game_state_mutex.question.answer_options =
                                        Some(shuffle_answers(&game_state_mutex.question))
                                }
                            }
                        }
                    },
                    GameStage::ResultsShow => {
                        if game_state_mutex.newgame_flag == true {
                            let new_game: GameState = GameState {
                                game_stage: GameStage::WaitingForPlayers,
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
                                question_stage: QuestionStage::QuestionIntroduction,
                                question_start_time: 0,
                                question_limit: 5,
                                players: vec![],
                                proceed_flag: false,
                                newgame_flag: false,
                                audio: None,
                                tts_text: None,
                                scores: vec![],
                            };
                            *game_state_mutex = new_game;
                        }
                    }
                    //GameStage::GameFinished => todo!(),
                }
            }
            thread::sleep(Duration::from_millis(250));
        }
    }

    fn count_players_answered_to_question(answers: &Vec<Answers>, question_number: u64) -> u64 {
        let mut num_answered: u64 = 0;

        for answer in answers.iter() {
            if answer.question_number == question_number {
                num_answered += 1;
            }
        }
        num_answered
    }

    fn shuffle_answers(question: &Question) -> Vec<String> {
        let mut options: Vec<String> = vec![];
        options.push(question.correct.clone());
        options.push(question.incorrect_1.clone());
        options.push(question.incorrect_2.clone());
        options.push(question.incorrect_3.clone());
        options.shuffle(&mut rand::rng());
        options
    }

    fn get_new_question(
        all_questions: &Root,
        questions: Arc<Mutex<Vec<Questions>>>,
        question_number: u64,
    ) -> Question {
        let mut rng = rand::rng();

        let mut questions_mutex = match questions.lock() {
            Ok(mutex) => mutex,
            Err(poisoned_mutex) => poisoned_mutex.into_inner(),
        };

        //dbg!(&questions_mutex);

        for _ in 0..all_questions.questions.len() {
            let random_question = rng.random_range(0..all_questions.questions.len()) as i64;
            let mut already_asked = false;
            for asked in questions_mutex.iter() {
                if asked.question_id == random_question {
                    already_asked = true;
                    break;
                }
            }
            if !already_asked {
                questions_mutex.push(Questions {
                    question_number: question_number,
                    question_id: random_question,
                });
                return all_questions.questions[random_question as usize].clone();
            }
        }

        // Unable to find question which has not been asked -- return first
        println!("PROBLEM! No more new questions!");
        all_questions.questions[0].clone()
    }

    fn count_points(
        all_questions: &Root,
        game_state_mutex: &GameState,
        questions: Arc<Mutex<Vec<Questions>>>,
        answers: &Vec<Answers>,
    ) -> Option<Vec<Points>> {
        let mut result: Vec<Points> = vec![];

        let questions_mutex = match questions.lock() {
            Ok(mutex) => mutex,
            Err(poisoned_mutex) => poisoned_mutex.into_inner(),
        };

        for player in game_state_mutex.players.iter() {
            println!("Calculating points for {}", player.name);
            let mut points: u32 = 0;
            for answer in answers.iter() {
                if answer.player_uuid == player.uuid {
                    //println!("Question number: {}", answer.question_number);
                    let correct_answer = get_correct_answer_for_question_id(
                        get_question_id_for_question_number(
                            answer.question_number,
                            &questions_mutex,
                        ),
                        all_questions,
                    );
                    //println!("Answer is {} -- correct: {}", answer.answer, correct_answer);
                    if answer.answer == correct_answer {
                        //println!("Correct answer! {}", answer.answer);
                        points += 1;
                    }
                }
            }
            result.push(Points {
                player_name: player.name.clone(),
                points: points,
            });
        }
        sort_results_by_points(result)
    }

    fn get_question_id_for_question_number(
        question_number: u64,
        questions: &Vec<Questions>,
    ) -> i64 {
        for question in questions.iter() {
            if question.question_number == question_number {
                //println!("{} --> {}", question_number, question.question_id);
                return question.question_id;
            }
        }
        dbg!(questions);
        return -999999;
    }

    fn get_correct_answer_for_question_id(question_id: i64, all_questions: &Root) -> String {
        for question in all_questions.questions.iter() {
            if question.id == question_id {
                return question.correct.clone();
            }
        }
        "-abcd".to_string()
    }

    fn sort_results_by_points(input: Vec<Points>) -> Option<Vec<Points>> {
        let mut result: Vec<Points> = vec![];
        let mut max: u32 = 0;
        let mut index = 0;
        let mut points = input.clone();
        dbg!(input);
        for _n in 0..points.len() {
            for (idx, player) in points.iter().enumerate() {
                if player.points > max {
                    max = player.points;
                    index = idx;
                }
            }
            result.push(points.get(index)?.clone());
            points.remove(index);
            max = 0;
        }
        dbg!(&result);
        Some(result)
    }
}
