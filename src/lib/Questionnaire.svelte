<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    let dispatch = createEventDispatcher()

    export let id: number
    export let src: AudioBuffer

    let answers = Array(4)
</script>

<div id="form">
    <div id="grid">
        <div class="question">
            <p class="prompt"><span class="qid">Q1</span> Riddle me this?</p>
            <div class="answer">
                <input type="text" />
            </div>
        </div>

        <div class="question">
            <p class="prompt"><span class="qid">Q2</span> Riddle me this?</p>
            <div class="answer">
                <input type="text" />
            </div>
        </div>

        <div class="question">
            <p class="prompt"><span class="qid">Q3</span> Riddle me this? Riddle me this? Riddle me this? Riddle me this? Riddle me this? Riddle me this? Riddle me this?</p>
            <div class="answer">
                <input type="text" />
            </div>
        </div>

        <div class="question">
            <p class="prompt"><span class="qid">Q4</span> Riddle me this?</p>
            <div class="answer">
                <label class="form-control">Option 1
                    <input type="radio" name="q4" />
                    <span class="radiomark"></span>
                </label>
                <label class="form-control">Second option
                    <input type="radio" name="q4" />
                    <span class="radiomark"></span>
                </label>
                <label class="form-control">Label C
                    <input type="radio" name="q4" />
                    <span class="radiomark"></span>
                </label>
                <label class="form-control">Fourth option
                    <input type="radio" name="q4" />
                    <span class="radiomark"></span>
                </label>
                <label class="form-control">Numero cinq
                    <input type="radio" name="q4" />
                    <span class="radiomark"></span>
                </label>
            </div>
        </div>

        <div class="question">
            <p class="prompt"><span class="qid">Q5</span> Riddle me this?</p>
            <div class="answer">
                <label class="form-control">Option 1
                    <input type="checkbox" name="q5" />
                    <span class="checkmark"></span>
                </label>
                <label class="form-control">Second option
                    <input type="checkbox" name="q5" />
                    <span class="checkmark"></span>
                </label>
                <label class="form-control">Label C
                    <input type="checkbox" name="q5" />
                    <span class="checkmark"></span>
                </label>
                <label class="form-control">Fourth option
                    <input type="checkbox" name="q5" />
                    <span class="checkmark"></span>
                </label>
                <label class="form-control">Numero cinq
                    <input type="checkbox" name="q5" />
                    <span class="checkmark"></span>
                </label>
            </div>
        </div>

        <div class="question">
            <p class="prompt"><span class="qid">Q6</span> Riddle me this?</p>
            <div class="answer">
                <input type="range" min={1} max={10} value={5} />
            </div>
        </div>
    </div>

    <button class="submit">Submit</button>

</div>

<style>
    #form {
        display: grid;
        height: 100%;
        grid-template-rows: 90% 10%;
        grid-template-columns: 100%;
        align-content: center;
        align-items: center;
        justify-content: center;
        justify-items: center;
    }

    #grid {
        display: flex;
        flex-wrap: wrap;
        justify-content: space-evenly;
        align-content: center;

        box-sizing: border-box;
        width: 100%;
        height: 100%;
        margin: auto;
    }

    .question {
        flex: 0 0 calc(46%);
        margin: 1rem 1rem;
        /*border: 2px solid purple;*/
    }

    .prompt {
        font-size: 2rem;
        text-align: left;
        margin: 1rem;
    }

    .qid {
        font-weight: 700;
        margin-right: 2rem;
    }

    .answer {
        width: 80%;
        margin: 0 auto 0 5rem;
        box-sizing: border-box;
        /*border: 2px solid blue;*/

        display: flex;
        flex-wrap: wrap;
        justify-content: center;
        align-content: center;
    }

    input {
        font-family: inherit;
        font-size: 1.6rem;
        text-align: left;
        line-height: 1.8rem;
        margin: 0.75rem 0.25rem 0 0.25rem;
        width: 100%;
        font-weight: 300;
    }

    input[type=text] {
        font-size: 2rem;
        border: 1px solid lightgrey;
    }

    .form-control {
        display: block;
        position: relative;
        padding: 0.5rem 0.5rem 0.5rem 2.3rem;
        cursor: pointer;
        font-size: 1.6rem;
        /*border: 1px solid #eeeeee;*/
        -webkit-user-select: none;
        -moz-user-select: none;
        -ms-user-select: none;
        user-select: none;
        margin: 0.2rem 0 0 0.6rem;
    }

    /* Hide the browser's default checkbox */
    .form-control input[type=checkbox] {
        position: absolute;
        opacity: 0;
        cursor: pointer;
        height: 0;
        width: 0;
    }

    /* Hide the browser's default radio button */
    .form-control input[type=radio] {
        position: absolute;
        opacity: 0;
        cursor: pointer;
    }

    /* Create a custom checkbox */
    .checkmark {
        position: absolute;
        top: 0.9rem;
        left: 0.5rem;
        height: 1.2rem;
        width: 1.2rem;
        background-color: #eee;
    }

    /* Create a custom radio button */
    .radiomark {
        position: absolute;
        top: 0.9rem;
        left: 0.5rem;
        height: 1.2rem;
        width: 1.2rem;
        background-color: #eee;
        border-radius: 50%;
    }

    /* On mouse-over, add a grey background color */
    .form-control:hover input[type=checkbox] ~ .checkmark {
        background-color: #ccc;
    }

    /* On mouse-over, add a grey background color */
    .form-control:hover input[type=radio] ~ .radiomark {
        background-color: #ccc;
    }

    /* When the checkbox is checked, add a blue background */
    .form-control input[type=checkbox]:checked ~ .checkmark {
        background-color: #2196F3;
        border: 1px solid dimgray;
    }

    /* When the radio button is checked, add a blue background */
    .form-control input[type=radio]:checked ~ .radiomark {
        background-color: #2196F3;
        border: 1px solid dimgray;
    }

    /* Create the checkmark/indicator (hidden when not checked) */
    .checkmark:after {
        content: "";
        position: absolute;
        display: none;
    }

    /* Create the indicator (the dot/circle - hidden when not checked) */
    .radiomark:after {
        content: "";
        position: absolute;
        display: none;
    }

    /* Show the checkmark when checked */
    .form-control input[type=checkbox]:checked ~ .checkmark:after {
        display: block;
    }

    /* Show the indicator (dot/circle) when checked */
    .form-control input[type=radio]:checked ~ .radiomark:after {
        display: block;
    }

    /* Style the checkmark/indicator */
    .form-control .checkmark:after {
        left: 0.35rem;
        top: -0.05rem;
        width: 0.4rem;
        height: 0.8rem;
        border: solid white;
        border-width: 0 0.2rem 0.2rem 0;
        -webkit-transform: rotate(45deg);
        -ms-transform: rotate(45deg);
        transform: rotate(45deg);
    }

    /* Style the indicator (dot/circle) */
    .form-control .radiomark:after {
        top: 0.35rem;
        left: 0.35rem;
        width: 0.5rem;
        height: 0.5rem;
        border-radius: 50%;
        background: white;
    }

    /* The slider itself */
    input[type=range] {
        -webkit-appearance: none;  /* Override default CSS styles */
        appearance: none;
        width: 100%; /* Full-width */
        height: 0.5rem; /* Specified height */
        background: #d3d3d3; /* Grey background */
        outline: none; /* Remove outline */
        opacity: 0.7; /* Set transparency (for mouse-over effects on hover) */
        -webkit-transition: .2s; /* 0.2 seconds transition on hover */
        transition: opacity .2s;
    }

    /* Mouse-over effects */
    input[type=range]:hover {
        opacity: 1; /* Fully shown on mouse-over */
    }

    /* The slider handle (use -webkit- (Chrome, Opera, Safari, Edge) and -moz- (Firefox) to override default look) */
    input[type=range]::-webkit-slider-thumb {
        -webkit-appearance: none; /* Override default look */
        appearance: none;
        width: 1.6rem; /* Set a specific slider handle width */
        height: 1.6rem; /* Slider handle height */
        background: #2196F3; /* Green background */
        border-radius: 50%; /* Circle handle */
        cursor: pointer; /* Cursor on hover */
    }

    input[type=range]::-moz-range-thumb {
        width: 1.6px; /* Set a specific slider handle width */
        height: 1.6px; /* Slider handle height */
        background: #2196F3; /* Green background */
        border-radius: 50%; /* Circle handle */
        cursor: pointer; /* Cursor on hover */
    }

    button {
        font-family: inherit;
        font-size: 1.6rem;
        line-height: 1.1rem;
        padding: 1.6rem 0.8rem;
        margin-top: 0.5rem;
        border-radius: 2em;
        border: 2px solid rgba(255, 62, 0, 0);
        outline: none;
        font-variant-numeric: tabular-nums;
        cursor: pointer;
    }

    button.submit {
        font-size: 1.7rem;
        font-weight: 500;
        width: 325px;
        color: white;
        background-color: rgba(0, 102, 255, 0.73);
    }

    button:focus {
        border: 2px solid #ff3e00;
    }

    button:active {
        /*border: 8px solid rgb(127, 218, 71);*/
        box-shadow: lightgray 0.1em 0.1em 0.1em 0.1em;
        font-weight: 600;
    }
</style>
