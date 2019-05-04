[![TravisCI build status](https://travis-ci.org/chris-zen/hero-studio.svg)](https://travis-ci.org/chris-zen/hero-studio)

# Hero Studio

This project is an attempt to build a music studio similar to Ableton Live or Bitwig Studio in the long term.

For the short term my main goal is to learn the core details about DAWs and audio programming, and may be to use the core engine to build more modest applications such as a VST host for development of plugins, a Raspberry Pi recording device, or a tracker.

## UI development

I plan to make this project very modular and keep UIs as separate applications. The plan is to have an engine that allows communication from external systems such a UI or a surface MIDI controllers, through some RPC mechanism (currently WebSockets + some serialization protocol).

I have been exploring several possibilities for building GUIs, more information can be found in [the wiki](https://github.com/chris-zen/hero-studio/wiki).

## Organisation

This is a multi-project composed by:

- [core](core): The core data model and logic.
- [app-server](app-server): The application server that runs the Audio/MIDI engine.
- [app-electron](app-electron): Very preliminary prototype of a GUI with Electron.

## Running

For running the main application you need to run the following commands:

```sh
cd app-server
cargo run --release
```

But bear in mind that you will need to install [PortAudio](http://www.portaudio.com/download.html) and [PortMIDI](http://portmedia.sourceforge.net/portmidi/) before running the project, as this is using the [portaudio](https://docs.rs/crate/portaudio/0.7.0) and [portmidi](https://crates.io/crates/portmidi) crates that depend on their binaries and headers.

If you get an error like:

```sh
error while loading shared libraries: libportaudio.so.2: cannot open shared object file: No such file or directory
```

then you should install the binary dependencies and headers for PortAudio and/or PortMIDI:

### Installing PortAudio & PortMIDI in Ubuntu

```sh
# ubuntu
sudo apt-get install libportaudio2 libportmidi0
```

### Installing PortAudio & PortMIDI in MacOS

```sh
brew install portaudio portmidi
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
- [x] app-server: Build MIDI output abstraction to send MIDI events to a device using CoreMIDI & PortMIDI as an initial impl
- [ ] app-server: Explore ways to avoid locks and allocations from the real-time processing threads (in progress)
- [ ] app-server: Add MIDI input support
- [ ] app-server: Add VST plugins support
- [ ] core: Filling MIDI output buffers from the Song clips (initially choose one type of clip between pianoroll, step-sequencer, drum-box).
- [ ] Start exploring UI integrations (some progress already done with Electron)
- [ ] ...
