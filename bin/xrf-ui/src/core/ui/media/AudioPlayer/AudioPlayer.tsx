import { Box } from "@mui/material";
import { ReactElement, SyntheticEvent, useCallback, useEffect, useRef, useState } from "react";

import { IMediaVolume, useMediaVolume } from "@/core/ui/media/use-media-volume";
import { Nullable } from "@/lib/types/general";

import { AudioPlayerControls } from "./AudioPlayerControls";
import { AudioWaveform } from "./AudioWaveform";

interface IAudioPlayerProps {
  src: string;
  bytes?: Nullable<Uint8Array>;
}

/** Transport for a single sound, following the media element's playback state. */
export function AudioPlayer({ src, bytes }: IAudioPlayerProps): ReactElement {
  const audioRef = useRef<Nullable<HTMLAudioElement>>(null);
  const volume: IMediaVolume = useMediaVolume();
  const [isPlaying, setPlaying] = useState<boolean>(false);
  const [isLooping, setLooping] = useState<boolean>(false);
  const [position, setPosition] = useState<number>(0);
  const [duration, setDuration] = useState<number>(0);

  const onTogglePlay = useCallback(() => {
    const audio: Nullable<HTMLAudioElement> = audioRef.current;

    if (!audio) {
      return;
    }

    if (audio.paused) {
      void audio.play().catch(() => undefined);
    } else {
      audio.pause();
    }
  }, []);

  const onSeek = useCallback(
    (next: number) => {
      const audio: Nullable<HTMLAudioElement> = audioRef.current;

      if (audio && duration > 0 && Number.isFinite(next)) {
        audio.currentTime = Math.min(duration, Math.max(0, next));
        // Repeated key presses must use the new position before the next media timeupdate event.
        setPosition(audio.currentTime);
      }
    },
    [duration]
  );

  const onToggleLoop = useCallback(() => setLooping((it: boolean) => !it), []);
  const onPlaying = useCallback(() => setPlaying(true), []);
  const onStopped = useCallback(() => setPlaying(false), []);

  const onTimeUpdate = useCallback((event: SyntheticEvent<HTMLAudioElement>) => {
    const next: number = event.currentTarget.currentTime;

    setPosition(Number.isFinite(next) ? Math.max(0, next) : 0);
  }, []);

  const onLoadedMetadata = useCallback((event: SyntheticEvent<HTMLAudioElement>) => {
    const next: number = event.currentTarget.duration;

    setDuration(Number.isFinite(next) ? Math.max(0, next) : 0);
  }, []);

  // Volume is a property of the element, not a rendered attribute.
  useEffect(() => {
    if (audioRef.current) {
      audioRef.current.volume = volume.value;
    }
  }, [src, volume.value]);

  // A source replacement stops playback without emitting pause.
  useEffect(() => {
    setPlaying(false);
    setPosition(0);
    setDuration(0);
  }, [src]);

  return (
    <Box sx={{ display: "flex", flexDirection: "column", gap: 1, width: "100%" }}>
      <Box
        component={"audio"}
        ref={audioRef}
        src={src}
        loop={isLooping}
        sx={{ display: "none" }}
        onPlay={onPlaying}
        onPause={onStopped}
        onEnded={onStopped}
        onTimeUpdate={onTimeUpdate}
        onLoadedMetadata={onLoadedMetadata}
      />

      <AudioWaveform
        src={src}
        bytes={bytes}
        position={position}
        duration={duration}
        onSeek={onSeek}
        onTogglePlay={onTogglePlay}
      />

      <AudioPlayerControls
        isPlaying={isPlaying}
        isLooping={isLooping}
        position={position}
        duration={duration}
        volume={volume.value}
        onTogglePlay={onTogglePlay}
        onToggleLoop={onToggleLoop}
        onVolumeChange={volume.set}
      />
    </Box>
  );
}
