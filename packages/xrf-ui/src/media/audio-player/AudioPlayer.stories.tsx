import type { Meta, StoryObj } from "@storybook/react-vite";
import { ReactElement } from "react";
import { fn } from "storybook/test";

import { useArgChange, withReadingColumn } from "#/testing/stories";

import { IAudioTransport, IAudioTransportState } from "./audio-transport";
import { AudioPlayer, IAudioPlayerProps } from "./AudioPlayer";

const PEAK_COUNT: number = 400;

/** A sound's envelope: a struck attack decaying under a slow swell, so the waveform reads as one. */
const PEAKS: Float32Array = Float32Array.from({ length: PEAK_COUNT }, (_, index: number) => {
  const time: number = index / PEAK_COUNT;

  return Math.min(1, Math.exp(-time * 4) * 0.9 + 0.25 * Math.abs(Math.sin(time * 37)) * (1 - time));
});

const STOPPED: IAudioTransportState = {
  playback: "stopped",
  position: 0,
  duration: 12.4,
  isLooped: false,
  failure: null,
};

/** A transport that only reports what it is asked: the story shows each state, nothing plays. */
function toTransport(state: IAudioTransportState): IAudioTransport {
  return { state, peaks: PEAKS, play: fn(), pause: fn(), seek: fn(), setLooped: fn() };
}

function toVolumeArgs(volume: number): Partial<IAudioPlayerProps> {
  return { volume };
}

function AudioPlayerStory(args: IAudioPlayerProps): ReactElement {
  const onVolumeChange = useArgChange(args.onVolumeChange, toVolumeArgs);

  return <AudioPlayer {...args} onVolumeChange={onVolumeChange} />;
}

const meta = {
  title: "Components/Media/AudioPlayer",
  component: AudioPlayer,
  render: AudioPlayerStory,
  decorators: [withReadingColumn],
  argTypes: {
    transport: { control: false },
    volume: { control: { type: "range", min: 0, max: 1, step: 0.05 } },
  },
  args: {
    transport: toTransport(STOPPED),
    volume: 0.8,
    onVolumeChange: fn(),
  },
} satisfies Meta<typeof AudioPlayer>;

export default meta;

type TStory = StoryObj<typeof meta>;

export const Stopped: TStory = {};

export const Playing: TStory = {
  args: { transport: toTransport({ ...STOPPED, playback: "playing", position: 4.2 }) },
};

export const Looped: TStory = {
  args: { transport: toTransport({ ...STOPPED, playback: "paused", position: 8.9, isLooped: true }) },
};

/** What the transport says when playing failed, above the waveform. */
export const Failed: TStory = {
  args: { transport: toTransport({ ...STOPPED, failure: "The sound is not an Ogg Vorbis stream." }) },
};

/** Peaks not known yet: the waveform waits for them. */
export const Decoding: TStory = {
  args: { transport: { ...toTransport(STOPPED), peaks: null } },
};
