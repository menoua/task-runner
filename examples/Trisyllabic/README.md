---
title: Instructions for running the Trisyllabic task
author:
    - Menoua Keshishian (mk4011@columbia.edu)
# date: \today{}
geometry: margin=3cm
header-includes:
    - \usepackage{setspace}
    - \spacing{1.4}
---

<!-- # Instructions for running the Trisyllabic task -->

## Description

The task consists of 3 5-minute blocks. Each block is a series of randomly ordered English syllables that will sometimes line up to form a proper 3-syllable English word.

## Subject

The subject should be asked to listen to the stimulus and identify the proper words within the stimulus.

To make sure they are paying attention, every time they hear the word "Objective", they should press the key provided to them. This key can be an external button connected to the recording system (preferred), or the keyboard of the computer running the task if no external button available (pressing any key other than "Escape" is fine, but "Space" is the preferred option since it's the largest).

At the end of each block there is a multiple choice question asking the subject which words they did not hear so far during the task. There is only one unspoken answer in each question, but 0 or multiple answers are also accepted.

## Running the task

Make sure the binary files (`neurotask-*`), the task file (`task.yml`), and the `resources` folder are all in the same directory. Then, to run the task open a terminal, go to the task directory using `cd`, and run the appropriate file (`./neurotask-macos` on macOS, and `./neurotask-linux` on Linux). Alternatively, on macOS, you can also simply double-click on the neurotask-macos file (you might need to "Right-click &rarr; Open" the first time instead of a double-click).

The 3 blocks should preferably be run in order (1&rarr;2&rarr;3). The completion of a block is indicated by a change of color.

To stop a block before it completes (started by mistake, loud interruption, etc.) you just need to press the "Escape" key twice in fast succession. If a block is stopped prematurely, you will need to start it from the beginning.

If using keyboard input instead of an external button, make sure the app stays in focus for the entire duration of the task (don't click outside the window after starting a block), so that all key presses by the subject are captured.

The task generates log files containing configuration, subject reactions and question responses, and stores them in a directory named "`output`".

## Configuration

There is one configuration option for the task---whether or not to use trigger signals. If unsure, do not change the default configuration.

There are two setups for the audio output:

1. Audio at left channel, trigger at right channel (default/preferred): In this case an audio left-right splitter should be used to separate the stimulus from the trigger and connect the stimulus to the speaker and the trigger to the recording system.

2. Audio at both channels: In this case a regular splitter should be used to connect the audio to both the speaker and the recording system.

## Issues to look out for

1. Linux only: if the program fails to start with an error containing the phrase "GraphicsAdapterNotFound", you are missing the Vulkan graphics drivers. For example, on Ubuntu, you can simply install them using:\
`sudo apt-get install libvulkan1`

2. If you hear intermittent clicks from the speaker, the trigger is present in the audio. This means either the splitter isn't working as expected, or if you didn't intend to use the trigger, the configuration needs to be changed.

3. For some reason, when using certain speakers the left and right channel of the output are flipped, i.e., right is audio, left is trigger. This seems to be speaker-specific, so if you are hearing the trigger only (intermittent clicks), just swap the channels.
