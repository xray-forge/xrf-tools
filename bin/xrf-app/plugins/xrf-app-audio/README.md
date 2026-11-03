# xrf-app-audio

The application's sound in the XRF desktop application: one `xrf_audio::AudioServer`, whose control thread owns the
engine on the output device, the settings it plays with, and the previews a domain opens for the page to play. Playing,
decoding, the device and the control thread are `xrf-audio`; this crate holds them for the application and answers the
page. Like the render plugin, it knows no domain: a domain plays its own sounds through these units, as the archives
plugin opens an archived sound as a preview.

## Layout

- `focus/`: `AudioFocus`, the windows that have the focus; it sets the server's gains at the settings' volumes, the
  master silent in the background where they say so.
- `settings/`: `AudioSettings` (device, `AudioVolumes`, background mute, and the gains they make) and
  `AudioSettingsStore`, which keeps them in the backend's preferences under `audio.settings`, so they hold before any
  page loads.
- `preview/`: `PreviewTransports`, the previews open on the server's Previews bus by `xrf_audio::AudioTransportId`, each
  with its window, its bytes and the page's channel; `PreviewEvents`, the server's event sink, hands a preview's changed
  state, pushed by the control thread (its position every 50 ms while it plays), to that channel.
- `commands/settings/`: the settings, the output's device and why none plays, and the system's devices.
- `commands/preview/`: playing, pausing, stopping, seeking, looping, watching, closing a preview, and its waveform.
- `plugin` and `ipc`: `AudioPlugin` and its command domain.

## In the application

`AudioPlugin` manages the `AudioServer`, `AudioFocus`, `AudioSettingsStore` and `PreviewTransports`, and answers the
`audio` domain. A window whose page loads again or that goes has its previews closed; the application mutes while no
window has the focus if the settings say so; the server shuts down as it exits. It runs no jobs.

## Checks

Run `cargo test -p xrf-app-audio` from the repository root.
