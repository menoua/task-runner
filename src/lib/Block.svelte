
<script lang="ts">
    import { createEventDispatcher } from 'svelte'
    let dispatch = createEventDispatcher()

    import Audio from './Audio.svelte'
    import { ActionInfo } from './types'

    import { resourceDir, join } from '@tauri-apps/api/path'
    const assetDir = resourceDir()

    import { readBinaryFile } from '@tauri-apps/api/fs'
    import Fixation from "./Fixation.svelte";
    const audioCtx = new AudioContext()

    export let actions: ActionInfo[]
    let done: boolean[] = new Array(actions.length)
    let position = Math.min(...actions.map(e => e.order))

    let start_time = new Date()

    let resources = new Map()
    let waitingOn = 0
    let finishedLoading = false
    for (let i = 0; i < actions.length; i++) {
        let type = actions[i].type
        if (type === Audio) {
            let path = actions[i].opt.src
            if (!resources.has(path)) {
                resources.set(path, null)
                waitingOn += 1
                load_audio(path).then(
                    source => {
                        resources.set(path, source)
                        waitingOn -= 1
                        console.log('Successfully loaded file: ' + path)// + ' @ ' + source.duration + 's')
                    },
                    error => alert('Failed to load audio file: ' + path)
                )
            }
        }
    }

    async function load_audio(path): Promise<AudioBuffer> {
        let buffer
        buffer = await join(await assetDir, 'assets', path)
        buffer = await readBinaryFile(buffer)
        buffer = await audioCtx.decodeAudioData(buffer.buffer)
        return buffer
    }

    function load_image(path) {

    }

    let interval = setInterval(() => {
        if (waitingOn === 0) {
            for (let i = 0; i < actions.length; i++) {
                if (actions[i].type === Audio) {
                    actions[i].opt.src = resources.get(actions[i].opt.src)
                }
            }
            clearInterval(interval)
            interval = null
            start_time = new Date()
            finishedLoading = true
        }
    }, 100)

    function end_action(event) {
        let id = event.detail
        done[id] = true

        let nextPosition = null
        for (let i = 0; i < actions.length; i++) {
            let order = actions[i].order
            let blocking = actions[i].blocking
            if (!done[i] && order >= position && blocking) {
                nextPosition = nextPosition === null ? order : Math.min(order, nextPosition)
            }
        }

        if (nextPosition === null) {
            let end_time = new Date()
            alert(((end_time - start_time) / 1000).toString() + ' seconds elapsed')
            dispatch('end')
        } else if (nextPosition !== position) {
            position = nextPosition
        }
    }
</script>

<div>
    {#if finishedLoading}
        {#each actions as { order, type, opt }, id}
            {#if position >= order && Math.floor(position) === Math.floor(order) && !done[id]}
                <svelte:component on:end={end_action} this={type} id={id} {...opt} />
            {/if}
        {/each}
    {:else}
        <Fixation />
    {/if}
</div>

<style>
    div {
        position: absolute;
        top: 0;
        right: 0;
        bottom: 0;
        left: 0;
        height: 100%;
        width: 100%;
    }
</style>
