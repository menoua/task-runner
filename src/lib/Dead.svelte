<script lang="ts">
    export let message = ''
</script>

<div>
    <h1>
        The application has stopped due to an error!
    </h1>

    <h2>
        {message}
    </h2>
</div>

<style>
    div {
        display: flex;
        flex-wrap: wrap;
        align-content: center;
        align-items: center;
        justify-content: center;
        justify-items: center;
        height: 100%;
        width: 100%;
    }

    h1 {
        color: red;
        text-align: center;
        margin: 1rem;
        width: 100%;
    }

    h2 {
        color: black;
        text-align: center;
        margin: 1rem;
        font-size: 1.8rem;
        width: 100%;
    }
</style>