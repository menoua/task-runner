<script lang="ts">
    import { onMount } from 'svelte'

    export let id: number
    export let size: number = 20

    let canvas

    onMount(() => {
        canvas.width  = canvas.offsetWidth;
        canvas.height = canvas.offsetHeight;
        const centerX = canvas.width / 2;
        const centerY = canvas.height / 2;

        let ctx = canvas.getContext('2d')
        ctx.lineWidth = 3

        ctx.moveTo(centerX - size, centerY);
        ctx.lineTo(centerX + size, centerY);
        ctx.stroke();

        ctx.moveTo(centerX, centerY - size);
        ctx.lineTo(centerX, centerY + size);
        ctx.stroke();
    })
</script>

<div>
    <canvas bind:this={canvas}></canvas>
</div>

<style>
    div {
        position: absolute;
        width: 100%;
        height: 100%
    }

    canvas {
        width: 100%;
        height: 100%;
    }
</style>
