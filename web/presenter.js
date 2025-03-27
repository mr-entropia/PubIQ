/* Set Ajax queries to synchronous */
$.ajaxSetup({
	async: false
});

var presenter_state = {};
var presenter_tick = "";
var narrator = "";
var audio_playing = false;
var last_audio_played = "";

function get_presenter_state(uuid)
{
	$.getJSON("/get_presenter_state/", function(data) {
		presenter_state = data;
        process_presenter_state();
	});
}

function command_to_game(command)
{
    console.log("Sending command to game: " + command);
    $.post("/command", { command: command }, function(data) {
        console.log("Response to command (" + command + ") is: " + data);
    });
}

function process_presenter_state() {
    if (presenter_state["game_stage"] == "WaitingForPlayers")
    {
        $("div#results").hide();
        $("span#count-players").html(presenter_state["num_players"]);
    }
    else if (presenter_state["game_stage"] == "IntroducePlayers")
    {
        if (presenter_state["audio"] == null) {
            command_to_game("proceed");
        } else {
            $("h2#player-intro").html(presenter_state["tts_text"]);
            $("div#waiting-for-players-presenter").hide();
            $("div#introduce-players").show();
            play_audio(presenter_state["audio"]);
        }
    }
    else if (presenter_state["game_stage"] == "GameInProgress")
    {
        if (presenter_state["question_stage"] == "QuestionIntroduction")
        {
            $("div#introduce-players").hide();
            $("div#question-answer").hide();
            $("h3#question").html(presenter_state["question"]);
            $("div#question").show();
            if (presenter_state["audio"] == null) {
                setTimeout(function() {
                    command_to_game("proceed");
                }, 5000);
            } else {
                play_audio(presenter_state["audio"]);
            }
        }
        else if (presenter_state["question_stage"] == "QuestionAnswerTime")
        {
            $("h4#answer-count").show();
            $("span#answer-count").html(presenter_state["num_players_answered"]);
        }
        else if (presenter_state["question_stage"] == "QuestionFinished")
        {
            $("div#question").hide();
            $("h3#answer").html(presenter_state["answer"]);
            $("h3#context").html(presenter_state["context"]);
            $("h4#answer-count").hide();
            $("div#question-answer").show();
            if (presenter_state["audio"] == null) {
                setTimeout(function() {
                    command_to_game("proceed");
                }, 5000);
            } else {
                play_audio(presenter_state["audio"]);
            }
        }
    }
    else if (presenter_state["game_stage"] == "ResultsShow")
    {
        $("div#question").hide();
        $("div#introduce-players").hide();
        $("div#question-answer").hide();
        $("h3#scores").html(presenter_state["scores"]);
        $("div#results").show();
        if (presenter_state["audio"] == null) {
            setTimeout(function() {
                command_to_game("proceed");
            }, 5000);
        } else {
            play_audio(presenter_state["audio"]);
        }
    }
}

function play_audio(path) {
    if (!audio_playing) {
        if (path != last_audio_played) {
            last_audio_played = path;
            audio_playing = true;
            console.log("Play audio: " + path);
            $("source#path").attr("src", path);
            narrator.load();
            narrator.play();
        }
    }
}

$(document).ready(function() {
    narrator = document.getElementById("narrator");
    narrator.addEventListener("ended", function() {
        $("#audio-finished").trigger("click");
        audio_playing = false;
    });

    $(document).on("click", "#btn-start-game", function() {
        command_to_game("proceed");
    });

    $(document).on("click", "#audio-finished", function() {
        console.log("Audio finished playing");
        setTimeout(function() {
            command_to_game("proceed");
        }, 3000);
    });

    presenter_tick = setInterval(function() { get_presenter_state(); }, 1000);
});