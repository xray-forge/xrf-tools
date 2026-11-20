import { AudioTransportState } from "@xrf/ipc/types/xrf-audio";
import { IAudioTransportState } from "@xrf/ui/media/audio-player";

/**
 * A preview's state as a player shows it: in seconds, where the backend sends milliseconds.
 */
export function toAudioTransportState(state: AudioTransportState): IAudioTransportState {
  return {
    playback: state.playback,
    position: state.position / 1000,
    duration: state.duration / 1000,
    isLooped: state.isLooped,
    failure: state.failure,
  };
}
