import { Nullable } from "@/lib/types/general";

/**
 * Names a channel count the way a sound editor would.
 *
 * Absent rather than zero when the stream header would not parse, which is a different thing from a sound that has no
 * channels - and the reason the count is optional at all.
 *
 * @param channels - Channels the stream header declares, or null when there is no readable header.
 * @returns Human readable channel description.
 */
export function formatAudioChannels(channels: Nullable<number>): string {
  switch (channels) {
    case null:
      return "-";

    case 1:
      return "1 (mono)";

    case 2:
      return "2 (stereo)";

    default:
      return String(channels);
  }
}
