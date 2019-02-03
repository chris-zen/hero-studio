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

You will need to install [PortAudio](http://www.portaudio.com/download.html) before running the project as we're using the [portaudio crate](https://docs.rs/crate/portaudio/0.7.0) which has a dependency on it.

If you get this error:

```sh
error while loading shared libraries: libportaudio.so.2: cannot open shared object file: No such file or directory
```

then you should install `libportaudio`:

```sh
# ubuntu
sudo apt-get install libportaudio2
```

Finally, you can run the project:

```sh
cd app-server
cargo run --release
```

## Tests

Tests can be run with:

```sh
cargo test
```

## Roadmap

- [x] core: Basic transport logic with accurate timing: play, stop, loop
- [x] app-native: Allow MIDI configuration through `studio.toml`
- [x] core: MIDI bus system
- [x] core: Implement a basic metronome using MIDI notes
- [x] core: Management of time and sync between different clocks: Audio, Midi, Host, Ticks
- [x] app-native: Build MIDI output abstraction to send MIDI events to a device (CoreMIDI and PortMIDI supported)
- [x] app-electron: Basic transport UI
- [x] app-server: Add a WebSockets server
- [ ] app-server: Refactoring the internal Audio/MIDI logic to be lockfree
- [ ] app-electron/app-server: synchronise UI and Audio/MIDI through WebSocket events
- [ ] core: Filling MIDI output buffers from the Song clips (initially choose one type of clip between pianoroll, step-sequencer, drum-box).
- [ ] ...
