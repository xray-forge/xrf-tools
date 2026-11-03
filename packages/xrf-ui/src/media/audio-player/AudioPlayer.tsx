import { ReactElement, useCallback } from "react";

import { Alert } from "#/material/alert";

import { IAudioTransport } from "./audio-transport";
import { AudioPlayerControls } from "./AudioPlayerControls";
import { AudioWaveform } from "./AudioWaveform";

interface IAudioPlayerProps {
  /** What plays the sound: the player shows its state and asks it to play, pause, seek and loop. */
  transport: IAudioTransport;
  /** Level in `[0, 1]`, which the caller keeps and applies. */
  volume: number;
  onVolumeChange: (volume: number) => void;
}

/** Transport controls, waveform and volume for a single sound, following whatever plays it. */
export function AudioPlayer({ transport, volume, onVolumeChange }: IAudioPlayerProps): ReactElement {
  const { state, peaks } = transport;

  const isPlaying: boolean = state.playback === "playing";

  const onTogglePlay = useCallback(() => {
    if (isPlaying) {
      transport.pause();
    } else {
      transport.play();
    }
  }, [isPlaying, transport]);

  const onToggleLoop = useCallback(() => transport.setLooped(!state.isLooped), [state.isLooped, transport]);

  const onSeek = useCallback((position: number) => transport.seek(position), [transport]);

  return (
    <div className={"flex w-full flex-col gap-2"}>
      {state.failure ? <Alert severity={"error"}>{state.failure}</Alert> : null}

      <AudioWaveform
        peaks={peaks}
        position={state.position}
        duration={state.duration}
        onSeek={onSeek}
        onTogglePlay={onTogglePlay}
      />

      <AudioPlayerControls
        isPlaying={isPlaying}
        isLooping={state.isLooped}
        position={state.position}
        duration={state.duration}
        volume={volume}
        onTogglePlay={onTogglePlay}
        onToggleLoop={onToggleLoop}
        onVolumeChange={onVolumeChange}
      />
    </div>
  );
}
