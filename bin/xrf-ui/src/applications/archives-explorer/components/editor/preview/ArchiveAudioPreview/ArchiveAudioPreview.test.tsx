import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RenderResult } from "@testing-library/react";

import { ArchiveAudioPreview } from "@/applications/archives-explorer/components/editor/preview/ArchiveAudioPreview";
import { ArchivesService } from "@/applications/archives-explorer/services/archives";
import { TArchiveBytes } from "@/core/archive";
import { AudioDescriptor } from "@/core/bindings/types/xrf-app";
import { mockInjectedService } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";
import { createLoadable } from "@/lib/loadable";
import { Nullable } from "@/lib/types/general";

const BYTES: TArchiveBytes = new Uint8Array([0x4f, 0x67, 0x67, 0x53]);

const FULL: AudioDescriptor = {
  channels: 2,
  sampleRate: 44100,
  parameters: { minDistance: 1, maxDistance: 50, baseVolume: 0.8, gameType: 3, maxAiDistance: 25 },
};

/**
 * Renders the panel over content the service already holds, which is the state the preview reads.
 *
 * @param descriptor - Sound description to publish, or null for a selection that produced no content.
 * @returns The render result.
 */
function renderPreview(descriptor: Nullable<AudioDescriptor>): RenderResult {
  const { service, container } = mockInjectedService(ArchivesService);

  service.content = createLoadable(descriptor ? { kind: "audio" as const, descriptor, bytes: BYTES } : null);

  return renderWithProviders(<ArchiveAudioPreview />, { container });
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
    const { getAllByText } = renderPreview({ channels: null, sampleRate: null, parameters: null });

    expect(getAllByText("-")).toHaveLength(2);
  });

  it("explains the engine defaults for a sound carrying no X-Ray comment", () => {
    const { getByText, queryByText } = renderPreview({ channels: 1, sampleRate: 22050, parameters: null });

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
