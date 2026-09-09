import { Alert, Box } from "@mui/material";
import { ReactElement, SyntheticEvent, useCallback, useEffect, useRef, useState } from "react";

import { IMediaVolume, useMediaVolume } from "@/core/ui/media/use-media-volume";
import { Nullable } from "@/lib/types/general";

import { describeAudioError, describePlaybackError } from "./AudioPlayer.errors";
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
  const [error, setError] = useState<Nullable<string>>(null);
  const playRequestRef = useRef<number>(0);

  const onTogglePlay = useCallback(() => {
    const audio: Nullable<HTMLAudioElement> = audioRef.current;

    if (!audio) {
      return;
    }

    const request: number = ++playRequestRef.current;
    const source: string = audio.src;

    if (audio.paused) {
      setError(null);

      if (audio.error) {
        audio.load();
      }

      void audio.play().catch((error: unknown) => {
        if (request === playRequestRef.current && audioRef.current === audio && audio.src === source) {
          setPlaying(false);
          setError(describePlaybackError(error));
        }
      });
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

  const onPlaying = useCallback(() => {
    setPlaying(true);
    setError(null);
  }, []);
  const onStopped = useCallback(() => setPlaying(false), []);

  const onError = useCallback((event: SyntheticEvent<HTMLAudioElement>) => {
    // The media error is more specific than the rejection of a pending play request.
    playRequestRef.current += 1;
    setPlaying(false);
    setError(describeAudioError(event.currentTarget.error));
  }, []);

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
    setError(null);

    return () => {
      // A replaced source or an unmounted player no longer owns a pending play rejection.
      playRequestRef.current += 1;
    };
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
        onError={onError}
      />

      {error ? <Alert severity={"error"}>{error}</Alert> : null}

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
