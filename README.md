# Hero Studio

This project is an attempt to build a music studio similar to Ableton Live or Bitwig Studio.

The initial scope will be to output only MIDI events, with the simplest way to arrange clips.

## UI development

All related to the UI is still very uncertain. I've been exploring several options but I don't feel like buying anyone of them yet. The one that I will be exploring consists on building the UI with Electron which would delegate to the core logic in rust. More information about the development of the idea [here](https://github.com/chris-zen/hero-studio/wiki/Electron-ideas).

Suggestions are welcome.

## Organisation

This is a multi-project composed by:

- [core](core): The core data model and logic.
- [app-native](app-native): The native application that handles Audio/MIDI and uses the core.

More sub-projects will come depending on how I decide to build the UI.

## Running

```
cd app-native
cargo run --release
```

## Tests

Tests can be run with:

```
cargo test
```

## Roadmap

[ ] core: Basic transport logic with accurate timing: play, stop, loop
[ ] core: Proper management of time and sync between different clocks: Audio, Midi, Host, Ticks
[ ] core: Implement a basic metronome using MIDI notes.
[ ] app-native: Build MIDI output abstraction to send MIDI events to a device using CoreMIDI as an initial impl.
[ ] core: Filling MIDI output buffers from the Song clips (initially choose one type of clip between pianoroll, step-sequencer, drum-box).
[ ] app-native: Allow MIDI configuration through `studio.toml`
[ ] Start exploring UI integration
[ ] ...
