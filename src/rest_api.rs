pub mod rest_http {
    use rouille::{post_input, router, try_or_400};
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    use crate::{
        game::state::{Answers, GameStage, GameState, Player, QuestionStage, Questions},
        helpers::time_helpers::uptime_ms,
    };

    pub fn run_rest_http_api(
        game_state: Arc<Mutex<GameState>>,
        _questions: Arc<Mutex<Vec<Questions>>>,
        answers: Arc<Mutex<Vec<Answers>>>,
    ) {
        let bind_address = "0.0.0.0:80";
        println!("REST API listening on {}", bind_address);

        rouille::start_server(bind_address, move |request| {
            // Check if static file is found
            let response = rouille::match_assets(request, "web");
            if response.is_success() {
                return response;
            }
            let response = rouille::match_assets(request, "web/audio");
            if response.is_success() {
                return response;
            }

            // If not, proceed to router
            router!(request,
                (GET) (/) => {
                    rouille::Response::redirect_302("/index.html")
                },

                (GET) (/version) => {
                    let s = format!("{}", json!({"version": "1.0"}));
                    rouille::Response::text(s)
                    .with_additional_header("Access-Control-Allow-Origin", "*")
                    .with_additional_header("Content-Type", "application/json")
                },

                (POST) (/register_player) => {
                    let player = try_or_400!(post_input!(request, {
                        name: String,
                    }));

                    match register_new_player(&game_state, &player.name) {
                        Ok(message) => {
                            return
                                rouille::Response::text(message)
                                .with_additional_header("Content-Type", "application/json");
                        },
                        Err(message) => {
                            return
                                rouille::Response::text(message)
                                .with_additional_header("Content-Type", "application/json");

                        },
                    }
                    rouille::Response::text("")
                },

                (GET) (/get_all_players) => {
                    let s = format!("{}", json!({"success": true, "players": get_all_players(&game_state)}).to_string());
                    rouille::Response::text(s)
                        .with_additional_header("Content-Type", "application/json")
                },

                (GET) (/get_player_state/{uuid: String}) => {
                    let s = format!("{}", get_player_state(&game_state, uuid));
                    rouille::Response::text(s)
                        .with_additional_header("Content-Type", "application/json")
                },

                (GET) (/get_presenter_state/) => {
                    let s = format!("{}", get_presenter_state(&game_state, &answers));
                    rouille::Response::text(s)
                        .with_additional_header("Content-Type", "application/json")
                },

                (POST) (/submit_answer) => {
                    let answer = try_or_400!(post_input!(request, {
                        uuid: String,
                        answer: String,
                    }));

                    match process_answer_submit(&game_state, &answers, &answer.uuid, &answer.answer) {
                        Ok(response) => {
                            rouille::Response::text(response)
                                .with_additional_header("Content-Type", "application/json")
                        },
                        Err(response) => {
                            rouille::Response::text(response)
                                .with_additional_header("Content-Type", "application/json")
                        },
                    }
                },

                (POST) (/command) => {
                    let command = try_or_400!(post_input!(request, {
                        command: String,
                    }));
                    match handle_presenter_command(&game_state, command.command) {
                        Ok(_) => {
                            rouille::Response::text(json!({"success": true}).to_string())
                                .with_additional_header("Content-Type", "application/json")
                        },
                        Err(error) => {
                            rouille::Response::text(json!({"success": false, "error": error}).to_string())
                                .with_additional_header("Content-Type", "application/json")
                        }
                    }
                },

                _ => rouille::Response::empty_404().
                with_additional_header("Access-Control-Allow-Origin", "*")
            )
        });
    }

    fn get_presenter_state(
        game_state: &Arc<Mutex<GameState>>,
        answers: &Arc<Mutex<Vec<Answers>>>,
    ) -> String {
        let game_state_mutex = match game_state.lock() {
            Ok(mutex) => mutex,
            Err(poisoned_mutex) => poisoned_mutex.into_inner(),
        };

        match game_state_mutex.game_stage {
            GameStage::WaitingForPlayers => {
                return json!({
                    "game_stage": game_state_mutex.game_stage.to_string(),
                    "num_players": game_state_mutex.players.len(),
                })
                .to_string();
            }
            GameStage::IntroducePlayers => {
                return json!({
                    "game_stage": game_state_mutex.game_stage.to_string(),
                    "num_players": game_state_mutex.players.len(),
                    "audio": game_state_mutex.audio,
                    "tts_text": game_state_mutex.tts_text,
                })
                .to_string();
            }
            GameStage::GameInProgress => {
                return json!({
                    "game_stage": game_state_mutex.game_stage.to_string(),
                    "question": game_state_mutex.question.question,
                    "question_stage": game_state_mutex.question_stage.to_string(),
                    "question_start_time": game_state_mutex.question_start_time,
                    "num_players": game_state_mutex.players.len(),
                    "num_players_answered": count_players_answered_to_question(answers, game_state_mutex.question_number).to_string(),
                    "answer": game_state_mutex.question.correct,
                    "context": game_state_mutex.question.context_information,
                    "audio": game_state_mutex.audio,
                }).to_string();
            }
            GameStage::ResultsShow => {
                return json!({
                    "game_stage": game_state_mutex.game_stage.to_string(),
                    "num_players": game_state_mutex.players.len(),
                    "audio": game_state_mutex.audio,
                    "tts_text": game_state_mutex.tts_text,
                    "scores": game_state_mutex.scores,
                })
                .to_string();
            }
            //GameStage::GameFinished => {
            //    return json!({
            //        "game_stage": game_state_mutex.game_stage.to_string(),
            //        "num_players": game_state_mutex.players.len(),
            //    }).to_string();
            //},
        }
    }

    fn count_players_answered_to_question(
        answers: &Arc<Mutex<Vec<Answers>>>,
        question_number: u64,
    ) -> u64 {
        let mut num_answered: u64 = 0;
        let answers_mutex = match answers.lock() {
            Ok(mutex) => mutex,
            Err(poisoned_mutex) => poisoned_mutex.into_inner(),
        };
        for answer in answers_mutex.iter() {
            if answer.question_number == question_number {
                num_answered += 1;
            }
        }
        num_answered
    }

    fn process_answer_submit(
        game_state: &Arc<Mutex<GameState>>,
        answers: &Arc<Mutex<Vec<Answers>>>,
        uuid: &String,
        answer: &String,
    ) -> Result<String, String> {
        let mut game_state_mutex = match game_state.lock() {
            Ok(mutex) => mutex,
            Err(poisoned_mutex) => poisoned_mutex.into_inner(),
        };

        let mut answers_mutex = match answers.lock() {
            Ok(mutex) => mutex,
            Err(poisoned_mutex) => poisoned_mutex.into_inner(),
        };

        let uuid = match Uuid::parse_str(&uuid) {
            Ok(uuid) => uuid,
            Err(_) => {
                return Err(json!({"success": false, "error": "Invalid UUID provided"}).to_string());
            }
        };

        let mut player_found = false;

        for player in game_state_mutex.players.iter_mut() {
            if player.uuid == uuid {
                player.last_seen = uptime_ms();
                player_found = true;
            }
        }

        if game_state_mutex.game_stage != GameStage::GameInProgress
            && game_state_mutex.question_stage != QuestionStage::QuestionAnswerTime
        {
            return Err(
                json!({"success": false, "error": "Answers not accepted at this time"}).to_string(),
            );
        }

        if !player_found {
            return Err(json!({"success": false, "error": "Invalid UUID provided"}).to_string());
        }

        for one_answer in answers_mutex.iter() {
            if one_answer.question_number == game_state_mutex.question_number
                && one_answer.player_uuid == uuid
            {
                return Err(
                    json!({"success": false, "error": "This question has already been answered"})
                        .to_string(),
                );
            }
        }

        answers_mutex.push(Answers {
            question_number: game_state_mutex.question_number,
            player_uuid: uuid,
            answer: answer.clone(),
        });

        dbg!(&answers_mutex);

        Ok(json!({"success": true}).to_string())
    }

    fn get_player_state(game_state: &Arc<Mutex<GameState>>, uuid: String) -> String {
        let mut game_state_mutex = match game_state.lock() {
            Ok(mutex) => mutex,
            Err(poisoned_mutex) => poisoned_mutex.into_inner(),
        };

        let uuid = match Uuid::parse_str(&uuid) {
            Ok(uuid) => uuid,
            Err(_) => {
                return json!({"success": false, "error": "Invalid UUID provided"}).to_string();
            }
        };

        let mut player_found = false;

        for player in game_state_mutex.players.iter_mut() {
            if player.uuid == uuid {
                player.last_seen = uptime_ms();
                player_found = true;
            }
        }

        if !player_found {
            return json!({"success": false, "error": "Invalid UUID provided"}).to_string();
        }

        match game_state_mutex.game_stage {
            crate::game::state::GameStage::WaitingForPlayers |
            crate::game::state::GameStage::ResultsShow |
            //crate::game::state::GameStage::GameFinished |
            crate::game::state::GameStage::IntroducePlayers => {
                let response = json!({
                    "success": true,
                    "game_stage": game_state_mutex.game_stage.to_string(),
                });
                return response.to_string();
            },
            crate::game::state::GameStage::GameInProgress => {
                let response = json!({
                    "success": true,
                    "game_stage": game_state_mutex.game_stage.to_string(),
                    "answer_options": game_state_mutex.question.answer_options,
                    "question_number": game_state_mutex.question_number,
                    "question_stage": game_state_mutex.question_stage.to_string(),
                    "question_start_time": game_state_mutex.question_start_time,
                });
                return response.to_string();
            },
        };
    }

    fn get_all_players(game_state: &Arc<Mutex<GameState>>) -> Vec<String> {
        let game_state_mutex = match game_state.lock() {
            Ok(mutex) => mutex,
            Err(poisoned_mutex) => poisoned_mutex.into_inner(),
        };

        let mut players: Vec<String> = vec![];

        for player in game_state_mutex.players.iter() {
            players.push(player.name.clone());
        }

        players
    }

    fn register_new_player(
        game_state: &Arc<Mutex<GameState>>,
        name: &String,
    ) -> Result<String, String> {
        let mut game_state_mutex = match game_state.lock() {
            Ok(mutex) => mutex,
            Err(poisoned_mutex) => poisoned_mutex.into_inner(),
        };

        // Check that game is not in progress
        if game_state_mutex.game_stage != GameStage::WaitingForPlayers {
            return Err(json!({"success": false, "error": "Game in progress"}).to_string());
        }

        // Check that player is not already registered
        for player in game_state_mutex.players.iter() {
            if player.name == *name {
                return Err(
                    json!({"success": false, "error": "Player already registered"}).to_string(),
                );
            }
        }

        let uuid = Uuid::new_v4();

        game_state_mutex.players.push(Player {
            name: name.clone(),
            uuid: uuid,
            last_seen: uptime_ms(),
            score: 0,
        });

        Ok(json!({"success": true, "uuid": uuid.to_string()}).to_string())
    }

    fn handle_presenter_command(
        game_state: &Arc<Mutex<GameState>>,
        command: String,
    ) -> Result<String, String> {
        let mut game_state_mutex = match game_state.lock() {
            Ok(mutex) => mutex,
            Err(poisoned_mutex) => poisoned_mutex.into_inner(),
        };

        match command.as_str() {
            "proceed" => {
                game_state_mutex.proceed_flag = true;
                return Ok(command);
            }
            "newgame" => {
                game_state_mutex.newgame_flag = true;
                return Ok(command);
            }
            _ => {
                return Err("Unknown command".to_string());
            }
        }
    }
}
