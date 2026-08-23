import { default as PauseIcon } from "@mui/icons-material/Pause";
import { default as PlayArrowIcon } from "@mui/icons-material/PlayArrow";
import { default as RepeatIcon } from "@mui/icons-material/Repeat";
import { default as VolumeUpIcon } from "@mui/icons-material/VolumeUp";
import { Box, IconButton, Slider, Tooltip, Typography, useTheme } from "@mui/material";
import {
  KeyboardEvent,
  MouseEvent,
  ReactElement,
  SyntheticEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

import { IMediaVolume, useMediaVolume } from "@/core/ui/media/use-media-volume";
import { extractPeaks, formatPlaybackTime } from "@/lib/media/waveform";
import { Nullable } from "@/lib/types/general";

/** One peak per two pixels: finer reads as noise, coarser loses short transients. */
const PEAKS_PER_PIXEL: number = 0.5;

const WAVEFORM_HEIGHT: number = 96;

export interface IAudioPlayerProps {
  src: string;
  bytes?: Nullable<Uint8Array>;
}

/**
 * Transport for a single sound, drawn rather than delegated to the browser's own widget.
 */
export function AudioPlayer({ src, bytes }: IAudioPlayerProps): ReactElement {
  const theme = useTheme();

  const audioRef = useRef<Nullable<HTMLAudioElement>>(null);
  const canvasRef = useRef<Nullable<HTMLCanvasElement>>(null);

  const volume: IMediaVolume = useMediaVolume();

  const [peaks, setPeaks] = useState<Nullable<Float32Array>>(null);
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
    (event: MouseEvent<HTMLCanvasElement>) => {
      const audio: Nullable<HTMLAudioElement> = audioRef.current;
      const bounds: Nullable<DOMRect> = canvasRef.current?.getBoundingClientRect() ?? null;

      if (!audio || !bounds || !duration) {
        return;
      }

      audio.currentTime = Math.min(duration, Math.max(0, ((event.clientX - bounds.left) / bounds.width) * duration));
    },
    [duration]
  );

  // Space is what every media surface binds, and the transport is useless from the keyboard without it.
  const onKeyDown = useCallback(
    (event: KeyboardEvent<HTMLCanvasElement>) => {
      if (event.key === " " || event.key === "Enter") {
        event.preventDefault();
        onTogglePlay();
      }
    },
    [onTogglePlay]
  );

  const onToggleLoop = useCallback(() => setLooping((it: boolean) => !it), []);

  const onPlaying = useCallback(() => setPlaying(true), []);

  const onStopped = useCallback(() => setPlaying(false), []);

  const onTimeUpdate = useCallback((event: SyntheticEvent<HTMLAudioElement>) => {
    setPosition(event.currentTarget.currentTime);
  }, []);

  const onLoadedMetadata = useCallback((event: SyntheticEvent<HTMLAudioElement>) => {
    setDuration(event.currentTarget.duration);
  }, []);

  const onChangeVolume = useCallback(
    (_: Event, next: number | Array<number>) => {
      volume.set(next as number);
    },
    [volume]
  );

  // Volume is not a rendered attribute, so it has to be written to the element itself - and written again for each new
  // source, because a remembered level would otherwise show on the slider while the sound played at full.
  useEffect(() => {
    if (audioRef.current) {
      audioRef.current.volume = volume.value;
    }
  }, [src, volume.value]);

  // Selecting another sound replaces the source silently - the element fires no `pause` for it - so the
  // transport would otherwise keep offering to pause a sound that already stopped, at its old length.
  useEffect(() => {
    setPlaying(false);
    setPosition(0);
    setDuration(0);
  }, [src]);

  // Decoding is best effort: a sound that will not decode here still plays, it simply has no picture.
  useEffect(() => {
    let isActive: boolean = true;

    setPeaks(null);

    if (!bytes || typeof AudioContext === "undefined") {
      return;
    }

    const context: AudioContext = new AudioContext();

    context
      .decodeAudioData(bytes.slice().buffer as ArrayBuffer)
      .then((buffer: AudioBuffer) => {
        if (isActive) {
          const width: number = canvasRef.current?.clientWidth ?? 600;

          setPeaks(extractPeaks(buffer.getChannelData(0), Math.max(1, Math.floor(width * PEAKS_PER_PIXEL))));
        }
      })
      .catch(() => undefined)
      .finally(() => void context.close().catch(() => undefined));

    return () => {
      isActive = false;
    };
  }, [bytes]);

  useEffect(() => {
    const canvas: Nullable<HTMLCanvasElement> = canvasRef.current;
    // jsdom has no 2d context, and neither does a canvas that has not been laid out yet.
    const canvasContext: Nullable<CanvasRenderingContext2D> = canvas?.getContext?.("2d") ?? null;

    if (!canvas || !canvasContext) {
      return;
    }

    const width: number = canvas.clientWidth;
    const height: number = canvas.clientHeight;

    canvas.width = width;
    canvas.height = height;

    canvasContext.clearRect(0, 0, width, height);

    const played: number = duration ? (position / duration) * width : 0;
    const middle: number = height / 2;
    const count: number = peaks?.length ?? 0;

    if (!count) {
      canvasContext.fillStyle = theme.palette.divider;
      canvasContext.fillRect(0, middle, width, 1);

      return;
    }

    for (let index = 0; index < count; index += 1) {
      const x: number = (index / count) * width;
      const magnitude: number = Math.max(1, (peaks as Float32Array)[index] * middle);

      canvasContext.fillStyle = x <= played ? theme.palette.primary.main : theme.palette.text.disabled;
      canvasContext.fillRect(x, middle - magnitude, Math.max(1, width / count - 1), magnitude * 2);
    }
  }, [duration, peaks, position, theme]);

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

      <Box
        component={"canvas"}
        ref={canvasRef}
        role={"slider"}
        tabIndex={0}
        aria-label={"Seek"}
        aria-valuemin={0}
        aria-valuemax={Math.round(duration)}
        aria-valuenow={Math.round(position)}
        sx={{
          width: "100%",
          height: WAVEFORM_HEIGHT,
          cursor: "pointer",
          borderRadius: 1,
          backgroundColor: "background.default",
          outline: "none",
        }}
        onClick={onSeek}
        onKeyDown={onKeyDown}
      />

      <Box sx={{ display: "flex", alignItems: "center", gap: 1 }}>
        <Tooltip describeChild title={isPlaying ? "Pause" : "Play"}>
          <IconButton aria-label={isPlaying ? "Pause" : "Play"} color={"primary"} onClick={onTogglePlay}>
            {isPlaying ? <PauseIcon /> : <PlayArrowIcon />}
          </IconButton>
        </Tooltip>

        <Typography variant={"caption"} className={"monospace"} sx={{ color: "text.secondary" }}>
          {formatPlaybackTime(position)} / {formatPlaybackTime(duration)}
        </Typography>

        <Box sx={{ flexGrow: 1 }} />

        <Tooltip describeChild title={"Loop"}>
          <IconButton
            aria-label={"Loop"}
            aria-pressed={isLooping}
            color={isLooping ? "primary" : "default"}
            size={"small"}
            onClick={onToggleLoop}
          >
            <RepeatIcon fontSize={"small"} />
          </IconButton>
        </Tooltip>

        <VolumeUpIcon fontSize={"small"} sx={{ color: "text.secondary" }} />

        <Slider
          aria-label={"Volume"}
          size={"small"}
          min={0}
          max={1}
          step={0.01}
          value={volume.value}
          sx={{ width: 96 }}
          onChange={onChangeVolume}
        />
      </Box>
    </Box>
  );
}
