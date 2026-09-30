import { clamp } from "@xrf/math";

/**
 * Reduces raw samples to one peak per horizontal pixel.
 *
 * A sound is tens of thousands of samples wide and a strip is a few hundred pixels, so drawing every
 * sample would be both slow and meaningless. Taking the loudest magnitude in each bucket keeps
 * transients visible - averaging instead smears a gunshot into the noise floor around it.
 *
 * @param samples - Mono channel data, as decoded from the sound.
 * @param buckets - Number of peaks to produce, normally the pixel width of the strip.
 * @returns One peak per bucket, each the loudest magnitude within it, always `buckets` long.
 */
export function extractPeaks(samples: Float32Array, buckets: number): Float32Array {
  const peaks: Float32Array = new Float32Array(Math.max(0, buckets));

  if (!samples.length || buckets <= 0) {
    return peaks;
  }

  const perBucket: number = samples.length / buckets;

  for (let bucket = 0; bucket < buckets; bucket += 1) {
    const start: number = Math.floor(bucket * perBucket);
    const end: number = clamp(Math.floor((bucket + 1) * perBucket), start + 1, samples.length);

    let peak: number = 0;

    for (let index = start; index < end; index += 1) {
      const magnitude: number = Math.abs(samples[index]);

      if (magnitude > peak) {
        peak = magnitude;
      }
    }

    peaks[bucket] = peak;
  }

  return peaks;
}

/**
 * Formats a playback position as `mm:ss`.
 *
 * @param seconds - Position or duration in seconds, as a media element reports it.
 * @returns Zero padded `mm:ss`, falling back to `00:00` for the `NaN` an unloaded element reports.
 */
export function formatPlaybackTime(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) {
    return "00:00";
  }

  const whole: number = Math.floor(seconds);

  return `${String(Math.floor(whole / 60)).padStart(2, "0")}:${String(whole % 60).padStart(2, "0")}`;
}
