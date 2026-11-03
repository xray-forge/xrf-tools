import { beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult, waitFor } from "@testing-library/react";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { EAudioCommand } from "@xrf/ipc/commands/audio";
import { getMockChannels, mockInvoke, mockIpcCommands, resetMockChannels } from "@xrf/ipc/testing";
import { AudioDescriptor } from "@xrf/ipc/types/xrf-app-archives";
import { AudioTransportState } from "@xrf/ipc/types/xrf-audio";
import { Nullable } from "@xrf/types";

import { mockArchiveFileDescriptor } from "@/core/archive/fixtures/archive.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";
import { ArchiveAudioPreview } from "@/tools/archives-explorer/components/editor/preview/ArchiveAudioPreview";
import {
  IArchivesExplorerMock,
  mockArchivesExplorer,
  mockShownArchiveFile,
} from "@/tools/archives-explorer/fixtures/archives-explorer.mocks";

const SESSION: string = "fixture-session";
const NAME: string = "sounds\\ambient\\wind.ogg";
const TRANSPORT: number = 7;

const FULL: AudioDescriptor = {
  channels: 2,
  sampleRate: 44100,
  parameters: { minDistance: 1, maxDistance: 50, baseVolume: 0.8, gameType: 3, maxAiDistance: 25 },
};

const PLAYING: AudioTransportState = {
  playback: "playing",
  position: 9000,
  duration: 65000,
  isLooped: false,
  failure: null,
};

function renderPreview(descriptor: Nullable<AudioDescriptor>): RenderResult {
  const explorer: IArchivesExplorerMock = mockArchivesExplorer();

  mockShownArchiveFile(
    explorer,
    mockArchiveFileDescriptor({ name: NAME }),
    descriptor ? { kind: "audio" as const, descriptor, name: NAME, sessionId: SESSION } : null
  );

  return renderWithProviders(<ArchiveAudioPreview />, { container: explorer.container });
}

describe("ArchiveAudioPreview", () => {
  beforeEach(() => {
    resetMockChannels();
    mockIpcCommands({
      [EArchivesCommand.OPEN_AUDIO_PREVIEW]: TRANSPORT,
      [EAudioCommand.WATCH_PREVIEW]: null,
      [EAudioCommand.READ_PREVIEW_PEAKS]: { peaks: [0.5, 1], duration: 65000 },
      [EAudioCommand.CLOSE_PREVIEW]: undefined,
    });
  });

  it("reports the stream the sound declares", () => {
    const { getByText } = renderPreview(FULL);

    expect(getByText("2 (stereo)")).toBeTruthy();
    expect(getByText("44100 Hz")).toBeTruthy();
  });

  it("says nothing rather than zero when the stream header would not parse", () => {
    // Zero channels at zero hertz is a claim about the sound; the backend reports absence precisely so the panel does
    // not have to make one.
    const { getAllByText } = renderPreview({
      channels: null,
      sampleRate: null,
      parameters: null,
    });

    expect(getAllByText("-")).toHaveLength(2);
  });

  it("explains the engine defaults for a sound carrying no X-Ray comment", () => {
    const { getByText, queryByText } = renderPreview({
      channels: 1,
      sampleRate: 22050,
      parameters: null,
    });

    expect(getByText("1 (mono)")).toBeTruthy();
    expect(queryByText("Min distance")).toBeNull();
    expect(getByText(/built-in source defaults/)).toBeTruthy();
  });

  it("opens the sound as a preview, follows its state, and closes it as it goes", async () => {
    const { findByText, unmount } = renderPreview(FULL);

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.OPEN_AUDIO_PREVIEW, { sessionId: SESSION, name: NAME })
    );
    await waitFor(() => expect(getMockChannels()).toHaveLength(1));

    act(() => getMockChannels()[0].onmessage(PLAYING));

    expect(await findByText("00:09 / 01:05")).toBeTruthy();

    unmount();

    expect(mockInvoke).toHaveBeenCalledWith(EAudioCommand.CLOSE_PREVIEW, { transport: TRANSPORT });
  });

  it("says why the sound could not be previewed", async () => {
    mockIpcCommands({
      [EArchivesCommand.OPEN_AUDIO_PREVIEW]: () => {
        throw new Error("Cannot preview the sound: not a sound");
      },
    });

    const { findByRole } = renderPreview(FULL);

    expect(await findByRole("alert")).toHaveTextContent("not a sound");
  });

  it("shows nothing to preview when the sound was not read", () => {
    const { getByText } = renderPreview(null);

    expect(getByText("Preview unavailable")).toBeTruthy();
    expect(mockInvoke).not.toHaveBeenCalledWith(EArchivesCommand.OPEN_AUDIO_PREVIEW, expect.anything());
  });
});
