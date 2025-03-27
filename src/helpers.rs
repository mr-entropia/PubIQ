pub mod time_helpers {
    /// Return machine uptime with millisecond-resolution (Unix-like systems)
    #[cfg(target_family = "unix")]
    pub fn uptime_ms() -> u64 {
        let uptime = std::time::Duration::from(
            nix::time::clock_gettime(nix::time::ClockId::CLOCK_MONOTONIC).unwrap(),
        )
        .as_millis();
        // FIXME: remove unwrap
        return uptime as u64;
    }

    /// Return machine uptime with millisecond-resolution (Windows systems)
    #[cfg(target_family = "windows")]
    pub fn uptime_ms() -> u64 {
        match uptime_lib::get() {
            Ok(uptime) => {
                return uptime.as_millis() as u64;
            }
            Err(err) => {
                eprintln!("Uptime ms error! {}", err);
                panic!();
            }
        }
    }
}

pub mod natural_language {
    pub fn get_player_names_for_tts(players: Vec<String>) -> String {
        let output = match players.len() {
            1 => players[0].clone(),
            2.. => {
                let mut output = "".to_string();
                let mut n = 0;
                for player in players.iter() {
                    if n == 0 {
                        output = player.to_string();
                    } else if n < players.len() - 1 {
                        output = format!("{}, {}", output, player);
                    } else {
                        output = format!("{} ja {}", output, player);
                    }
                    n += 1;
                }
                output
            }
            _ => "".to_string(),
        };
        output
    }

    pub fn correct_answer_and_context_announcement(correct: &String, context: &String) -> String {
        format!("{} {}", correct, context)
    }

    pub fn prompt_for_player_introduction(players: String) -> String {
        format!("Olet tietovisaisäntä. Tietovisa on nimeltään Pub I Q, ja se on juuri alkamassa. Esittele pelaajat {}. Toivota heille hyvää onnea. Vastaus voi olla korkeintaan neljä lausetta pitkä.", players)
    }

    pub fn prompt_for_winner_announcement(winner: String, num_points: String) -> String {
        format!("Olet tietovisaisäntä. Tietovisa on juuri päättynyt ja voittajaksi on selviytynyt pelaaja nimeltä {}. Hän keräsi {} pistettä! Onnittele voittajaa sekä kiitä kaikkia osallistujia pelistä. Käytä ylitsevuotavaista hehkutusta, jos mahdollista. Vastaus voi olla enintään neljä lausetta pitkä.", winner, num_points)
    }
}
