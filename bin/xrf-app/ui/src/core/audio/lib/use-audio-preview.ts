import { Channel } from "@tauri-apps/api/core";
import { audioCommands } from "@xrf/ipc/commands/audio";
import { AudioTransportId, AudioTransportState, SoundPeaks } from "@xrf/ipc/types/xrf-audio";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";
import { IAudioTransport, IAudioTransportState } from "@xrf/ui/media/audio-player";
import { useEffect, useEffectEvent, useMemo, useState } from "react";

import { toAudioTransportState } from "@/core/audio/lib/audio-transport-state";

const log: Logger = new Logger(__MODULE_NAME__);

/** Peaks read of a preview's sound: one per two pixels of a strip 4096 pixels wide, reduced to the strip's own width. */
const PREVIEW_PEAKS: number = 2048;

/** Where a preview stands once it is open: its id, what it last said of itself, and its waveform. */
interface IOpenPreview {
  id: AudioTransportId;
  state: Nullable<IAudioTransportState>;
  peaks: Nullable<Float32Array>;
}

export interface IAudioPreviewOptions {
  /** Names the sound: another key closes the open preview and opens the next. */
  key: string;
  /** Opens the sound as a preview through the domain that holds it. */
  open: () => Promise<AudioTransportId>;
}

export interface IAudioPreview {
  /** What a player plays the sound through, or null until the preview is open and has said where it stands. */
  transport: Nullable<IAudioTransport>;
  /** Why the preview could not be opened or steered, or null. */
  error: Nullable<string>;
}

/**
 * Plays one sound through the application's audio as a preview: opens it, follows its state on a channel, reads its
 * waveform, and closes it when the key changes or the component goes, a preview that opens too late included.
 *
 * @param options - The sound's key and how it is opened, or null while there is no sound to preview.
 * @returns The transport a player plays it through, and why it failed.
 */
export function useAudioPreview(options: Nullable<IAudioPreviewOptions>): IAudioPreview {
  const [preview, setPreview] = useState<Nullable<IOpenPreview>>(null);
  const [error, setError] = useState<Nullable<string>>(null);

  const key: Nullable<string> = options?.key ?? null;

  const transport: Nullable<IAudioTransport> = useMemo(() => {
    if (!preview?.state) {
      return null;
    }

    const id: AudioTransportId = preview.id;

    function steer(request: Promise<unknown>): void {
      void request.catch((failure: unknown) => {
        log.warn("A preview refused a request:", failure);
        setError(String(failure));
      });
    }

    return {
      state: preview.state,
      peaks: preview.peaks,
      play: () => steer(audioCommands.playPreview(id)),
      pause: () => steer(audioCommands.pausePreview(id)),
      seek: (position: number) => steer(audioCommands.seekPreview(id, Math.round(Math.max(0, position) * 1000))),
      setLooped: (isLooped: boolean) => steer(audioCommands.setPreviewLooped(id, isLooped)),
    };
  }, [preview]);

  const onOpen = useEffectEvent((): Nullable<Promise<AudioTransportId>> => options?.open() ?? null);

  useEffect(() => {
    let isActive: boolean = true;
    let opened: Nullable<AudioTransportId> = null;

    setPreview(null);
    setError(null);

    const opening: Nullable<Promise<AudioTransportId>> = onOpen();

    /** Changes the preview this effect opened, and never one a later run of it opened. */
    function update(change: (current: IOpenPreview) => IOpenPreview): void {
      setPreview((current: Nullable<IOpenPreview>) => (current && current.id === opened ? change(current) : current));
    }

    if (!opening) {
      return;
    }

    opening
      .then(async (id: AudioTransportId) => {
        if (!isActive) {
          await audioCommands.closePreview(id);

          return;
        }

        const states: Channel<AudioTransportState> = new Channel<AudioTransportState>();

        opened = id;
        states.onmessage = (state: AudioTransportState): void => {
          if (isActive) {
            update((current: IOpenPreview) => ({ ...current, state: toAudioTransportState(state) }));
          }
        };

        setPreview({ id, state: null, peaks: null });

        await audioCommands.watchPreview(id, states);

        const read: SoundPeaks = await audioCommands.readPreviewPeaks(id, PREVIEW_PEAKS);

        if (isActive) {
          update((current: IOpenPreview) => ({
            ...current,
            peaks: Float32Array.from(read.peaks, (peak: Nullable<number>) => peak ?? 0),
          }));
        }
      })
      .catch((failure: unknown) => {
        if (isActive) {
          log.warn("Could not preview the sound:", failure);
          setError(String(failure));
        }
      });

    return () => {
      isActive = false;

      if (opened !== null) {
        void audioCommands
          .closePreview(opened)
          .catch((failure: unknown) => log.warn("Could not close a preview:", failure));
      }
    };
  }, [key]);

  return { transport, error };
}
