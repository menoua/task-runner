<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    let dispatch = createEventDispatcher()

    export let id: number
    export let src: AudioBuffer

    const audioCtx = new AudioContext()
    let source = audioCtx.createBufferSource()
    source.buffer = src
    source.onended = () => dispatch('end', id)
    source.connect(audioCtx.destination)
    source.start()
</script>
