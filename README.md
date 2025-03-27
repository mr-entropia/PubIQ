# PubIQ
## Introduction
PubIQ is a multiplayer pub quiz. Players play with their phones, tablets or computers. One computer, preferably connected to a TV and audio system, works as the presenter. Presenter's screen will show questions while players screens only show answer options.

At the moment PubIQ is fully playable but only barely so. **It's in proof-of-concept state.**

## Technical details
* Players are implemented as a Single Page App with HTML, CSS and jQuery
* Presenter's front-end is implemented as a Single Page App with HTML, CSS and jQuery
* Presenter's back-end is implemented using Rust
* Questions are read from a JSON file
* Generative AI features have been integrated
* Since Elevenlabs' service only has pretty limited free tier, speech for questions and answers is only generated once and then cached. To re-generate a speech, delete appropriate MP3 file from `web/audio/`. `q-xy` or `a-xy`, where `q` = question, `a` = answer, `xy` = question/answer number.
* A question helper tool has been included. Use it to generate JSON and then copy-paste that to `questions.json`.

## Generative AI features
* Google Gemini 2.0 Flash is used to generate an introductory text, as well as winner announcement text
* Elevenlabs Eleven Flash v2.5 model is used to synthesize all speech, i.e. introduction, questions, answer context as well as winner announcement

## How to get started
* Get API keys for Google Gemini 2.0 Flash as well as Elevenlabs
* Set API keys as environment variables called `ELEVENLABS_API_KEY` and `GOOGLE_GENAI_STUDIO_API_KEY`
* Compile Rust code and start it
* Point presenter to `http://host-address/presenter.html` and all players to `http://host-address/`
* Once players have given their names, presenter can start the game

