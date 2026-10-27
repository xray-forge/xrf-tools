import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";
import { AudioDescriptor } from "@xrf/ipc/types/xrf-app-archives";
import { Nullable } from "@xrf/types";

import { mockArchiveFileDescriptor } from "@/core/archive/fixtures/archive.mocks";
import { TArchiveBytes } from "@/core/archive/lib";
import { renderWithProviders } from "@/fixtures/utils/render";
import { ArchiveAudioPreview } from "@/tools/archives-explorer/components/editor/preview/ArchiveAudioPreview";
import {
  IArchivesExplorerMock,
  mockArchivesExplorer,
  mockShownArchiveFile,
} from "@/tools/archives-explorer/fixtures/archives-explorer.mocks";

const BYTES: TArchiveBytes = new Uint8Array([0x4f, 0x67, 0x67, 0x53]);

const FULL: AudioDescriptor = {
  channels: 2,
  sampleRate: 44100,
  parameters: { minDistance: 1, maxDistance: 50, baseVolume: 0.8, gameType: 3, maxAiDistance: 25 },
  mediaType: "audio/ogg",
};

function renderPreview(descriptor: Nullable<AudioDescriptor>): RenderResult {
  const explorer: IArchivesExplorerMock = mockArchivesExplorer();

  mockShownArchiveFile(
    explorer,
    mockArchiveFileDescriptor(),
    descriptor ? { kind: "audio" as const, descriptor, bytes: BYTES } : null
  );

  return renderWithProviders(<ArchiveAudioPreview />, { container: explorer.container });
}

describe("ArchiveAudioPreview", () => {
  beforeEach(() => {
    jest.spyOn(URL, "createObjectURL").mockImplementation(() => "blob:sound");
    jest.spyOn(URL, "revokeObjectURL").mockImplementation(() => undefined);
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
      mediaType: "audio/ogg",
    });

    expect(getAllByText("-")).toHaveLength(2);
  });

  it("explains the engine defaults for a sound carrying no X-Ray comment", () => {
    const { getByText, queryByText } = renderPreview({
      channels: 1,
      sampleRate: 22050,
      parameters: null,
      mediaType: "audio/ogg",
    });

    expect(getByText("1 (mono)")).toBeTruthy();
    expect(queryByText("Min distance")).toBeNull();
    expect(getByText(/built-in source defaults/)).toBeTruthy();
  });

  it("plays the bytes it was handed rather than fetching them again", () => {
    const { container } = renderPreview(FULL);

    expect(container.querySelector("audio")?.getAttribute("src")).toBe("blob:sound");
  });

  it("shows nothing to preview when the sound was not read", () => {
    const { getByText } = renderPreview(null);

    expect(getByText("Preview unavailable")).toBeTruthy();
  });
});
