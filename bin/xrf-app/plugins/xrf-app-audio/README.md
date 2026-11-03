# xrf-app-audio

The application's sound in the XRF desktop application: one audio engine on the output device, the settings it plays
with, and the previews a domain opens for the page to play. Playing, decoding and the device are `xrf-audio`; this
crate holds them for the application and answers the page. Like the render plugin, it knows no domain: a domain plays
its own sounds through these units, as the archives plugin opens an archived sound as a preview.

## Layout

- `engine/`: `AudioEngineState`, the engine and its output under one lock, playing at the settings' device and volumes
  and silent in the background where they say so.
- `settings/`: `AudioSettings` (device, `AudioVolumes`, background mute) and `AudioSettingsStore`, which keeps them in
  the backend's preferences under `audio.settings`, so they hold before any page loads.
- `preview/`: `PreviewTransports`, the previews open, each an `xrf_audio::AudioTransport` on the Previews bus addressed by
  an `AudioTransportId`; a watched preview's state reaches its page's channel whenever it changes, every 50 ms at most.
- `commands/settings/`: the settings, the output's device and why none plays, and the system's devices.
- `commands/preview/`: playing, pausing, stopping, seeking, looping, watching, closing a preview, and its waveform.
- `plugin` and `ipc`: `AudioPlugin` and its command domain.

## In the application

`AudioPlugin` manages `AudioEngineState`, `AudioSettingsStore` and `PreviewTransports`, and answers the `audio` domain.
A page that loads again finds its previews closed; the application mutes while no window has the focus if the settings
say so; previews close as it exits. It runs no jobs.

## Checks

Run `cargo test -p xrf-app-audio` from the repository root.
