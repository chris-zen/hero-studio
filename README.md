# Hero Studio

This project is an attempt to build a music studio similar to Ableton Live or Bitwig Studio.

The initial scope will be to output only MIDI events, with the simplest way to arrange clips.

## UI development

All related to the UI is still very uncertain. I've been exploring several options but I don't feel like buying anyone of them yet.
- Building the UI in Electron and delegate all the logic to the rust core using some type of RPC or IPC.
- Native UI solutions in rust, but they seem quite unmature ([azul.rs](http://azul.rs), [conrod](https://github.com/PistonDevelopers/conrod), [imgui-rs](https://github.com/Gekkio/imgui-rs), [nuklear-rust](https://github.com/snuk182/nuklear-rust/issues?q=is%3Aopen+is%3Aissue), [qt QML](https://archive.fosdem.org/2018/schedule/event/rust_qt_binding_generator/)).
- Compiling the core into wasm and using HTML5/CSS/JavaScript/WebAudio/WebMIDI for the frontend (I wouldn't be able to use VST or AudioUnits).
- UI written in C++ but linking with the rust core for everything else.

My main concerns/requirements:
- Performance: There will be plenty of visual indicators changuing state all the time (vu meters, leds, scrolling clips, ...)
- Layouting: The layout of the UI will be complex and I don't won't to make layout calculations myself.
- Canvas vs Vector graphics: Ideally it integrates vector graphics support easily, but canvas rendering would also be ok.
- Communication/linking with the rust logic: There are plenty of Web options for UI design, and quite some mature ones in C++. But the question is how how to link them to my rust core logic. RPC, IPC, cross-linking, ...)
- Custom widgets: It should support creating custom components/widgets as well as customise very common ones such as buttons, labels and so on.

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
