import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { act, renderHook } from "@testing-library/react";
import { EAudioCommand } from "@xrf/ipc/commands/audio";
import { getMockChannels, mockInvoke, mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { AudioTransportId, AudioTransportState, SoundPeaks } from "@xrf/ipc/types/xrf-audio";

import { IAudioPreviewOptions, useAudioPreview } from "@/core/audio/lib/use-audio-preview";

const PEAKS: SoundPeaks = { peaks: [0.5, null], duration: 2000 };

function stateAt(position: number): AudioTransportState {
  return { playback: "playing", position, duration: 2000, isLooped: false, failure: null };
}

function getClosed(): Array<unknown> {
  return mockInvoke.mock.calls
    .filter(([name]) => name === EAudioCommand.CLOSE_PREVIEW)
    .map(([, args]) => (args as { transport: AudioTransportId }).transport);
}

// A task, so it runs after every promise the open, watch and peaks requests chain.
async function settle(): Promise<void> {
  await act(async () => new Promise((resolve) => setTimeout(resolve)));
}

function previewOf(key: string, id: AudioTransportId): IAudioPreviewOptions {
  return { key, open: async () => id };
}

describe("useAudioPreview", () => {
  beforeEach(() => {
    mockIpcCommands({ [EAudioCommand.READ_PREVIEW_PEAKS]: PEAKS });
  });

  afterEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockInvoke.mockClear();
  });

  it("follows the open preview's state and closes it when the key changes", async () => {
    const { result, rerender } = renderHook((options: IAudioPreviewOptions) => useAudioPreview(options), {
      initialProps: previewOf("first", 1),
    });

    await settle();
    act(() => getMockChannels()[0].onmessage(stateAt(500)));

    expect(result.current.transport?.state.position).toBe(0.5);

    rerender(previewOf("second", 2));
    await settle();

    expect(getClosed()).toEqual([1]);
    expect(result.current.transport).toBeNull();
  });

  it("closes a preview that opens after its key is gone, and never follows it", async () => {
    const late: { open: (id: AudioTransportId) => void } = { open: () => {} };
    const { unmount } = renderHook(() =>
      useAudioPreview({ key: "late", open: () => new Promise((resolve) => (late.open = resolve)) })
    );

    unmount();
    await act(async () => late.open(3));

    expect(getClosed()).toEqual([3]);
    expect(mockInvoke.mock.calls.some(([name]) => name === EAudioCommand.WATCH_PREVIEW)).toBe(false);
  });

  it("ignores what a closed preview's channel still sends", async () => {
    const { result, rerender } = renderHook((options: IAudioPreviewOptions) => useAudioPreview(options), {
      initialProps: previewOf("first", 1),
    });

    await settle();
    rerender(previewOf("second", 2));
    await settle();
    act(() => getMockChannels()[1].onmessage(stateAt(250)));
    act(() => getMockChannels()[0].onmessage(stateAt(1500)));

    expect(result.current.transport?.state.position).toBe(0.25);
  });
});
