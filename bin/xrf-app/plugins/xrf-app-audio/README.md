# xrf-app-audio

Sound in the XRF desktop application: one `xrf_audio::AudioServer` on the output device, with Steam Audio acoustics,
the settings it plays with, and the previews a domain opens for the page to play. Playing, decoding, the device and the
control thread are `xrf-audio`; this crate holds them for the application and answers the page. It knows no domain: a
domain plays its own sounds through these units, as the archives plugin opens an archived sound as a preview.

## Layout

- `focus/`: `AudioFocus`, the windows that have the focus; it sets the server's gains from the settings' volumes,
  silencing the master in the background when the settings say so.
- `settings/`: `AudioSettings` (device, `AudioVolumes`, background mute, and the gains they make) and
  `AudioSettingsStore`, which keeps them in the backend's preferences under `audio.settings`, so they apply before any
  page loads.
- `preview/`: `PreviewTransports`, the previews open on the server's Previews bus by `xrf_audio::AudioTransportId`, each
  with its window, its bytes and the page's channel; and `PreviewEvents`, the server's event sink, which forwards a
  preview's changed state to that channel (its position every 50 ms while it plays).
- `commands/settings/`: the settings, the output's device and why none plays, and the system's devices.
- `commands/preview/`: playing, pausing, stopping, seeking, looping, watching and closing a preview, and its waveform.
- `plugin` and `ipc`: `AudioPlugin` and its command domain.

## In the application

`AudioPlugin` manages the `AudioServer`, `AudioFocus`, `AudioSettingsStore` and `PreviewTransports`, and answers the
`audio` domain. It runs no jobs.

- A window's previews close when its page reloads or the window closes.
- The application mutes while no window has the focus, if the settings say so.
- Steam Audio loads from the packaged resources, else from the checkout's `.sdks/steam-audio`.
- The server shuts down when the application exits.

## Checks

Run `cargo test -p xrf-app-audio` from the repository root.
