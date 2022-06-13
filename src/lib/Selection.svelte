<script lang="ts" xmlns="http://www.w3.org/1999/html">
    import { createEventDispatcher } from 'svelte'
    const dispatch = createEventDispatcher()

    export let blocks
</script>

<h1>Select a block:</h1>

<div class="grid">
    {#each blocks as {name, done}, id}
        <button on:click={() => dispatch('start', id)} class="item" class:done={done}>
            {name}
        </button>
    {/each}
</div>

<style>
    h1 {
        /*color: #ff3e00;*/
        color: #000000;
        font-size: 2.5rem;
        font-weight: 300;
        line-height: 1.5;
        padding-top: 1rem;
        padding-bottom: 2rem;
        margin: auto;
    }

    .item {
        font-family: inherit;
        font-size: 1.75rem;
        font-weight: 300;
        line-height: 1.3;
        padding: 0.8em;
        color: black;
        background-color: white;
        border-radius: 2.5em;
        border: 4px solid rgb(127, 218, 71);
        outline: none;
        font-variant-numeric: tabular-nums;
        cursor: pointer;

        flex: 0 0 calc(27.5%);
        height: 125px;
        margin: 1.5rem 1.5rem;
    }

    .item.done {
        color: black;
        background-color: rgb(167, 238, 126);
    }

    .item:focus {
        /*border: 2px solid #ff3e00;*/
    }

    .item:active {
        /*background-color: rgba(255, 62, 0, 0.2);*/
        border: 8px solid rgb(127, 218, 71);
        box-shadow: lightgray 0.1em 0.1em 0.1em 0.1em;
        font-weight: 500;
        padding: 0.6em;
    }

    .grid {
        display: flex;
        flex-wrap: wrap;
        justify-content: center;
        align-content: center;

        box-sizing: border-box;
        width: 90%;
        height: 85%;
        margin: auto;
    }
</style>
