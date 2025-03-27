/* Set Ajax queries to synchronous */
$.ajaxSetup({
	async: false
});

var uuid = "";
var game_state = {};
var game_tick = "";
var override_game = false;

function get_player_state(uuid)
{
	$.getJSON("/get_player_state/" + uuid, function(data) {
		game_state = data;
        process_player_state();
	});
}

function submit_answer(uuid, answer)
{
    $.post("/submit_answer", { uuid: uuid, answer: answer }, function(data) {
        console.log(data);
    });
}

function register_player(name)
{
    $.post("/register_player", { name: name }, function(data) {
        if (data.success == true) {
            uuid = data["uuid"];
            console.log("uuid: " + uuid);
            $("div#join-game").fadeOut("slow", function() {
                $("div#waiting-for-players").fadeIn("slow");
                game_tick = setInterval(function() { get_player_state(uuid); }, 1000);
            });
        } else {
            alert("Ei voitu liittyä peliin!\n\n" + data["error"]);
        }
    });
}

function process_player_state() {
    //console.log(game_state);
    if (game_state["game_stage"] == "GameInProgress")
    {
        $("div#waiting-for-players").hide();
        if (game_state["question_stage"] == "QuestionIntroduction" || game_state["question_stage"] == "QuestionFinished")
        {
            override_game = false;
            $("div#game").hide();
            $("div#look-at-tv").show();
        } else {
            if (!override_game)
            {
                $("div#look-at-tv").hide();
                $("div#game").show();
                $("button#btn-answer-one").html(game_state["answer_options"][0]);
                $("button#btn-answer-two").html(game_state["answer_options"][1]);
                $("button#btn-answer-three").html(game_state["answer_options"][2]);
                $("button#btn-answer-four").html(game_state["answer_options"][3]);
            }
        }
    }
    else if (game_state["game_stage"] == "ResultsShow")
    {
        console.log("results show");
    }
}

$(document).ready(function() {
    $(document).on("click", "#btn-register-player", function() {
        register_player($("input#player-name").val());
    });

    $(document).on("click", "#btn-answer-one", function() {
        submit_answer(uuid, $(this).html());
        override_game = true;
        $("div#game").fadeOut("slow", function() {
            $("div#look-at-tv").fadeIn("slow");
        });
    });

    $(document).on("click", "#btn-answer-two", function() {
        submit_answer(uuid, $(this).html());
        override_game = true;
        $("div#game").fadeOut("slow", function() {
            $("div#look-at-tv").fadeIn("slow");
        });
    });

    $(document).on("click", "#btn-answer-three", function() {
        submit_answer(uuid, $(this).html());
        override_game = true;
        $("div#game").fadeOut("slow", function() {
            $("div#look-at-tv").fadeIn("slow");
        });
    });

    $(document).on("click", "#btn-answer-four", function() {
        submit_answer(uuid, $(this).html());
        override_game = true;
        $("div#game").fadeOut("slow", function() {
            $("div#look-at-tv").fadeIn("slow");
        });
    });
});