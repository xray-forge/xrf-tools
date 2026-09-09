/** Explains a failed playback request without exposing browser implementation details. */
export function describePlaybackError(error: unknown): string {
  if (error instanceof Error) {
    switch (error.name) {
      case "NotAllowedError":
        return "Audio playback was blocked. Press Play to try again.";
      case "NotSupportedError":
        return "This audio format is not supported or the file is damaged.";
      case "AbortError":
        return "Audio playback was interrupted. Press Play to try again.";
    }
  }

  return "Could not play this audio. Press Play to try again.";
}

/** Explains the media element's read or decode failure. */
export function describeAudioError(error: MediaError | null): string {
  switch (error?.code) {
    case 1:
      return "Audio loading was interrupted. Press Play to try again.";
    case 2:
      return "Could not read this audio. Press Play to try again.";
    case 3:
      return "Could not decode this audio. The file may be damaged or use an unsupported format.";
    case 4:
      return "This audio format is not supported or the file is unavailable.";
    default:
      return "Could not load this audio. Press Play to try again.";
  }
}
