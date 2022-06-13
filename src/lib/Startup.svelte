<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    const dispatch = createEventDispatcher()

    export let stereo
    export let trigger
    export let subjectId
    export let instructions
</script>

<table style="width: 100%; height: 100%; alignment: center; margin: auto; padding: 2%">
    <tr>
        <td colspan="8">
            <h1>Trisyllabic</h1>
        </td>
    </tr>

    <tr>
        <td colspan="8">
            <svelte:component this={instructions} />
        </td>
    </tr>

    <tr>
        <td style="width: 5%"></td>
        <td style="width: 15%">
            <button on:click={() => stereo = !stereo} class="config" style="width: 175px">
                { stereo ? 'Stereo' : 'Mono' }
            </button>
        </td>
        <td style="width: 20%">
            <button on:click={() => trigger = !trigger} class="config" style="width: 250px">
                { trigger ? 'with' : 'without' } Trigger
            </button>
        </td>
        <td style="width: 5%"></td>
        <td style="width: 15%">
            <label>
                Subject ID:<br/>
                <input type="text" bind:value={subjectId} minlength="3" maxlength="12" />
            </label>
        </td>
        <td style="width: 5%"></td>
        <td style="width: 25%">
            <button on:click|once={() => dispatch('start')} class="start">
                START
            </button>
        </td>
        <td style="width: 5%"></td>
    </tr>
</table>

<style>
    h1 {
        color: #ff3e00;
        font-size: 2.5rem;
        font-weight: 200;
        line-height: 1.2;
        margin: auto;
    }

    label {
        font-family: inherit;
        font-size: 1.6rem;
        line-height: 1.8rem;
        font-weight: 400;
    }

    input[type=text] {
        font-family: inherit;
        font-size: 1.6rem;
        text-align: center;
        line-height: 1.8rem;
        margin: 0.75rem 0.25rem 0 0.25rem;
        width: 12rem;
        font-weight: 300;
    }

    button {
        font-family: inherit;
        font-size: 1.6rem;
        line-height: 1.6rem;
        padding: 1.2em 0.8em;
        border-radius: 2em;
        border: 2px solid rgba(255, 62, 0, 0);
        outline: none;
        font-variant-numeric: tabular-nums;
        cursor: pointer;
    }

    button.config {
        width: 225px;
        color: black;
        background-color: rgba(211,211,211,0.25);
    }

    button.start {
        font-size: 1.7rem;
        font-weight: 500;
        width: 325px;
        color: white;
        background-color: rgba(0, 102, 255, 0.73);
    }

    button:focus {
        border: 2px solid #ff3e00;
    }

    button.config:active {
        background-color: rgba(194, 193, 193, 0.25);
    }

    button:active {
        /*border: 8px solid rgb(127, 218, 71);*/
        box-shadow: lightgray 0.1em 0.1em 0.1em 0.1em;
        font-weight: 600;
    }
</style>
