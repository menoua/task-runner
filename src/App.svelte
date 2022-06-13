<script lang="ts">
  import Startup from './lib/Startup.svelte'
  import Selection from './lib/Selection.svelte'
  import Block from './lib/Block.svelte'
  import Audio from './lib/Audio.svelte'
  import Counter from './lib/Counter.svelte'
  import Fixation from './lib/Fixation.svelte'
  import Dead from './lib/Dead.svelte'
  import { BlockInfo } from './lib/types.ts'
  import {setupBlockDir, setupOutputDir} from './lib/utils'

  import { exit } from '@tauri-apps/api/process'

  import Instructions from './assets/Instructions.svelte'
  import Questionnaire from './lib/Questionnaire.svelte'

  enum State {
    Starting,
    Selecting,
    Running,
    Dead,
  }

  let date = new Date()
  let stereo = false
  let trigger = true
  let subjectId = String(date.getFullYear()) + '-'
          + String(date.getMonth()+1).padStart(2, '0') + '-'
          + String(date.getDate()).padStart(2, '0')
  let projectId = 'Trisyllabic'
  let errorMessage = ''
  let outputDir = null
  let activeDir = null

  let state = State.Starting
  let activeBlock = null

  let lastEsc = new Date()

  let blocks: BlockInfo[] = [
    {
      name: 'Block 1',
      actions: [
        { order: 0, type: Counter, blocking: true },
        { order: 0, type: Counter, blocking: true },

        { order: 1, type: Counter, blocking: true },
        { order: 1, type: Counter, blocking: false },

        { order: 2, type: Audio, blocking: true, opt: { src: 'sample-6s.mp3' } },
        { order: 2, type: Counter, blocking: true },

        { order: 3, type: Audio, blocking: false, opt: { src: 'sample-6s.mp3' } },
        { order: 3, type: Counter, blocking: true },

        { order: 4, type: Audio, blocking: true, opt: { src: 'sample-6s.mp3' } },
        { order: 4, type: Counter, blocking: false },

        { order: 5, type: Fixation, blocking: false, opt: { size: 20 } },
        { order: 5, type: Audio, blocking: true, opt: { src: 'sample-6s.mp3' } },
      ],
      done: false,
    },

    {
      name: 'Block 2',
      actions: [
        { order: 0, type: Counter, blocking: true, opt: {} },
        { order: 0, type: Audio, blocking: true, opt: { src: 'sample-6s.mp3' } }
      ],
      done: false,
    },

    {
      name: 'Block 3',
      actions: [
        { order: 0, type: Counter, blocking: true, opt: {} },
        { order: 1, type: Counter, blocking: true, opt: {} },
      ],
      done: false,
    },

    {
      name: 'Block 4',
      actions: Array(101),
      done: false,
    },

    {
      name: 'Block 5',
      actions: [
        { order: 0, type: Audio, blocking: true, opt: { src: 'sample-6s.mp3' } },
      ],
      done: false,
    },

    {
      name: 'Semantic embedding',
      actions: [
        { order: 0, type: Questionnaire, blocking: true, opt: { } },
      ],
      done: false,
    },

    {
      name: 'Semantic dissimilarity',
      actions: [],
      done: false,
    },
  ]

  for (let i = 0; i < 100; i++) {
    blocks[3].actions[i] = { order: 1.0 + i/100, type: Audio, blocking: true, opt: { src: 'sample-6s.mp3' } }
  }
  blocks[3].actions[100] = { order: 1.0, type: Fixation, blocking: false }

  function handleKeydown(event) {
    let key = event.key
    // let keyCode = event.keyCode

    if (key === 'Escape') {
      event.preventDefault()
      let currEsc = new Date()
      if (currEsc - lastEsc > 250) {
        lastEsc = currEsc
        return
      }

      if (state === State.Running) {
        kill_block()
      } else {
        exit(0)
      }
    } else {
      // log `key` to file here
    }
  }

  async function start_task() {
    await setupOutputDir(projectId, subjectId).then(
      dir => {
        alert('Saving output to directory:\n' + dir)
        outputDir = dir
        state = State.Selecting
      },
      err => {
        errorMessage = err
        state = State.Dead
      },
    )
  }

  async function start_block(event) {
    let blockId = event.detail
    await setupBlockDir(outputDir, blockId).then(
      dir => {
        activeDir = dir
        activeBlock = blockId
        state = State.Running
      },
      err => {
        errorMessage = err
        state = State.Dead
      }
    )
  }

  function end_block() {
    blocks[activeBlock].done = true
    state = State.Selecting
    activeBlock = null
    activeDir = null
  }

  function kill_block() {
    state = State.Selecting
    activeBlock = null
    activeDir = null
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<main>
  {#if state === State.Starting}
    <Startup on:start={start_task}
             bind:stereo={stereo}
             bind:trigger={trigger}
             bind:subjectId={subjectId}
             instructions={Instructions} />
  {:else if state === State.Selecting}
    <Selection on:start={start_block}
               bind:blocks={blocks} />
  {:else if state === State.Running}
    <Block on:end={end_block}
           bind:actions={blocks[activeBlock].actions} />
  {:else}
    <Dead bind:message={errorMessage} />
  {/if}
</main>

<style>
  :root {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen,
      Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  }

  main {
    text-align: center;
    padding: 1em;
    margin: auto;
    height: 70%;
    width: 75%;
    border: 1px dashed black;
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0
  }
</style>
