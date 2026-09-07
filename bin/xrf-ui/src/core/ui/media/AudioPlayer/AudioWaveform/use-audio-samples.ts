import { useEffect, useState } from "react";

import { Nullable, Optional } from "@/lib/types/general";

interface IAudioSamples {
  src: string;
  bytes: Uint8Array;
  samples: Float32Array;
}

/**
 * Decodes waveform samples independently of playback, discarding results for replaced sources.
 *
 * @param src - Identity of the selected sound.
 * @param bytes - Encoded sound bytes, when available.
 * @returns The current sound's first channel, or null while unavailable or decoding.
 */
export function useAudioSamples(src: string, bytes: Optional<Nullable<Uint8Array>>): Nullable<Float32Array> {
  const [decoded, setDecoded] = useState<Nullable<IAudioSamples>>(null);

  useEffect(() => {
    let isActive: boolean = true;

    setDecoded(null);

    if (!bytes || typeof AudioContext === "undefined") {
      return;
    }

    const sourceBytes: Uint8Array = bytes;

    async function decode(): Promise<void> {
      let context: Nullable<AudioContext> = null;

      try {
        context = new AudioContext();

        const buffer: AudioBuffer = await context.decodeAudioData(new Uint8Array(sourceBytes).buffer);

        if (isActive) {
          setDecoded({ src, bytes: sourceBytes, samples: buffer.getChannelData(0) });
        }
      } catch {
        // A refused context or unsupported encoding leaves playback available without a waveform.
      } finally {
        await context?.close().catch(() => undefined);
      }
    }

    void decode();

    return () => {
      isActive = false;
    };
  }, [src, bytes]);

  return decoded?.src === src && decoded.bytes === bytes ? decoded.samples : null;
}
