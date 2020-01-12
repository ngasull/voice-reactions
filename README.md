# voice-reactions
Basic voice and video processing with SDL and ffmpeg: make the video play when someone speaks in the microphone.

## Build Dependencies

First, make sure to [have Rust toolchain installed](https://www.rust-lang.org/tools/install).

### Linux
- `libavcodec-dev`
- `libavformat-dev`
- `libavutil-dev`
- `libsdl2-dev`
- `libswresample-dev`
- `libswscale-dev`

### OSX
Not tested yet.

According to [these instructions](https://wiki.libav.org/Platform/MacOSX):
```
brew install yasm
brew install libav
```
And sdl (not sure if it includes dev libs):
```
brew install sdl
```

### Windows
Not tested yet.

Libs and dlls can be found here: https://ffmpeg.zeranoe.com/builds/

## Build using Rust

```
cargo build --release
```
