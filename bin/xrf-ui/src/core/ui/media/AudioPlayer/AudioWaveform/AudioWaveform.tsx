import { Box, useTheme } from "@mui/material";
import { KeyboardEvent, MouseEvent, ReactElement, useCallback, useLayoutEffect, useMemo, useRef } from "react";

import { extractPeaks, formatPlaybackTime } from "@/lib/media/waveform";
import { useElementSize } from "@/lib/react/use-element-size";
import { Nullable } from "@/lib/types/general";

import { useAudioSamples } from "./use-audio-samples";

/** One peak per two pixels preserves short transients without crowding the strip. */
const PEAKS_PER_PIXEL: number = 0.5;
const WAVEFORM_HEIGHT: number = 96;
/** Arrow keys move by seconds, independent of the waveform's pixel width. */
const SEEK_STEP: number = 5;

interface IAudioWaveformProps {
  src: string;
  bytes?: Nullable<Uint8Array>;
  position: number;
  duration: number;
  onSeek: (position: number) => void;
  onTogglePlay: () => void;
}

/** Resizable waveform and keyboard-accessible seek surface for the current sound. */
export function AudioWaveform({
  src,
  bytes,
  position,
  duration,
  onSeek,
  onTogglePlay,
}: IAudioWaveformProps): ReactElement {
  const theme = useTheme();
  const canvasRef = useRef<Nullable<HTMLCanvasElement>>(null);
  const [measure, size] = useElementSize<HTMLCanvasElement>();
  const samples: Nullable<Float32Array> = useAudioSamples(src, bytes);

  const length: number = Number.isFinite(duration) ? Math.max(0, duration) : 0;
  const current: number = Number.isFinite(position) ? Math.min(length, Math.max(0, position)) : 0;
  const width: number = size?.width ?? 0;
  const peaks: Nullable<Float32Array> = useMemo(
    () => (samples && width > 0 ? extractPeaks(samples, Math.max(1, Math.floor(width * PEAKS_PER_PIXEL))) : null),
    [samples, width]
  );

  const attach = useCallback(
    (canvas: Nullable<HTMLCanvasElement>) => {
      canvasRef.current = canvas;
      measure(canvas);
    },
    [measure]
  );

  function seek(next: number): void {
    if (length > 0) {
      onSeek(Math.min(length, Math.max(0, next)));
    }
  }

  function onClick(event: MouseEvent<HTMLCanvasElement>): void {
    const bounds: DOMRect = event.currentTarget.getBoundingClientRect();

    if (bounds.width > 0) {
      seek(((event.clientX - bounds.left) / bounds.width) * length);
    }
  }

  function onKeyDown(event: KeyboardEvent<HTMLCanvasElement>): void {
    switch (event.key) {
      case "ArrowLeft":
      case "ArrowDown":
        event.preventDefault();
        seek(current - SEEK_STEP);
        break;
      case "ArrowRight":
      case "ArrowUp":
        event.preventDefault();
        seek(current + SEEK_STEP);
        break;
      case "Home":
        event.preventDefault();
        seek(0);
        break;
      case "End":
        event.preventDefault();
        seek(length);
        break;
      case " ":
      case "Enter":
        event.preventDefault();
        onTogglePlay();
        break;
    }
  }

  useLayoutEffect(() => {
    const canvas: Nullable<HTMLCanvasElement> = canvasRef.current;
    const context: Nullable<CanvasRenderingContext2D> = canvas?.getContext?.("2d") ?? null;

    if (!canvas || !context || !size) {
      return;
    }

    canvas.width = size.width;
    canvas.height = size.height;
    context.clearRect(0, 0, size.width, size.height);

    const middle: number = size.height / 2;

    if (!peaks?.length) {
      context.fillStyle = theme.palette.divider;
      context.fillRect(0, middle, size.width, 1);

      return;
    }

    const played: number = length ? (current / length) * size.width : 0;

    for (let index = 0; index < peaks.length; index += 1) {
      const x: number = (index / peaks.length) * size.width;
      const magnitude: number = Math.max(1, peaks[index] * middle);

      context.fillStyle = x <= played ? theme.palette.primary.main : theme.palette.text.disabled;
      context.fillRect(x, middle - magnitude, Math.max(1, size.width / peaks.length - 1), magnitude * 2);
    }
  }, [size, peaks, current, length, theme]);

  return (
    <Box
      aria-label={"Seek"}
      aria-valuemin={0}
      aria-valuemax={length}
      aria-valuenow={current}
      aria-valuetext={`${formatPlaybackTime(current)} of ${formatPlaybackTime(length)}`}
      aria-disabled={length === 0}
      component={"canvas"}
      ref={attach}
      role={"slider"}
      tabIndex={0}
      sx={{
        width: "100%",
        height: WAVEFORM_HEIGHT,
        cursor: "pointer",
        borderRadius: 1,
        backgroundColor: "background.default",
        "&:focus-visible": { outline: "2px solid", outlineColor: "primary.main", outlineOffset: 2 },
      }}
      onClick={onClick}
      onKeyDown={onKeyDown}
    />
  );
}
