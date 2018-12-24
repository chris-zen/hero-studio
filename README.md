# Hero Studio

This project is an attempt to build a music studio similar to Ableton Live or Bitwig Studio.

The initial scope will be to output only MIDI events, with the simplest way to arrange clips.

All related to the UI is still very uncertain. I've been exploring several options but I don't feel like buying anyone of them:
- Native UI solutions in rust seem quite unmature ([azul.rs](http://azul.rs), [conrod](https://github.com/PistonDevelopers/conrod), [imgui-rs](https://github.com/Gekkio/imgui-rs), [nuklear-rust](https://github.com/snuk182/nuklear-rust/issues?q=is%3Aopen+is%3Aissue), [qt QML](https://archive.fosdem.org/2018/schedule/event/rust_qt_binding_generator/)).
- Compiling the core into wasm and using HTML5/CSS/JavaScript/WebAudio/WebMIDI for the frontend (I couldn't use VST or AudioUnits).
- UI written in C++ but linking with the rust core for everything else.


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
