import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";
import { isComputedProp, isObservableProp } from "@wirestate/mobx";

import { TextureEncodingComparison } from "@/core/bindings/types/xrf-app";
import { JobsService } from "@/core/jobs/services/jobs";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { MOCK_TEXTURE, mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";

import { TextureEncodingService } from "./texture-encoding.service";

function mockCandidate(format: "bc1" | "bc3", label: string) {
  return {
    channelRmse: [0, 0, 0, 0] as [number, number, number, number],
    compatibility: [],
    encodeDuration: 30,
    fileBytes: 1024,
    format,
    gpuBytes: 512,
    label,
    psnr: 42,
    supportSummary: "all renderers, GL unverified",
  };
}

function mockComparison(reference: string = MOCK_TEXTURE): TextureEncodingComparison {
  return {
    candidates: [mockCandidate("bc1", "BC1 (DXT1)"), mockCandidate("bc3", "BC3 (DXT5)")],
    current: { fileBytes: 2048, gpuBytes: 1024, height: 16, label: "DXT5", mipmapLevels: 1, width: 16 },
    outcome: "completed",
    reference,
  };
}

async function mockService(): Promise<{ service: TextureEncodingService; selection: TextureSelectionService }> {
  setMockInvokeResponses({
    ["plugin:textures|compare_encodings"]: mockComparison(),
    ["plugin:textures|describe"]: mockTextureDescription(),
    ["plugin:textures|read_candidate"]: new ArrayBuffer(8),
    ["plugin:textures|read_texture"]: new ArrayBuffer(0),
  });

  const container: Container = mockContainer([JobsService, TextureSelectionService, TextureEncodingService]);
  const selection: TextureSelectionService = container.get(TextureSelectionService);

  await selection.openFile("C:\\gamedata\\textures\\ston\\ston_beton05.dds");

  return { selection, service: container.get(TextureEncodingService) };
}

describe("TextureEncodingService", () => {
  beforeEach(() => resetMockInvoke());

  it("applies its mobx annotations", () => {
    // A service whose annotations never got applied still passes every behavioural test here, because nothing in jest
    // reacts to its state - and then does nothing at all in the running app.
    const container: Container = mockContainer([JobsService, TextureSelectionService, TextureEncodingService]);
    const service: TextureEncodingService = container.get(TextureEncodingService);

    expect(isObservableProp(service, "chosen")).toBe(true);
    expect(isComputedProp(service, "comparison")).toBe(true);
    expect(isComputedProp(service, "isDirty")).toBe(true);
  });

  it("weighs nothing until it is asked", async () => {
    // Five encodes, of which BC7 alone is over a second on a large texture. A person who opened the file to change a
    // flag should not pay for that.
    const { service } = await mockService();

    expect(service.comparison).toBeNull();
    expect(service.isDirty).toBe(false);
  });

  it("holds a comparison for the texture it was made for", async () => {
    const { service } = await mockService();

    await service.run("kaiser");

    expect(service.comparison?.candidates).toHaveLength(2);
  });

  it("shows no comparison against a texture it was not made for", async () => {
    // The backend keeps one comparison at a time and the figures are about specific pixels, so a comparison made for
    // another file is not an answer about this one.
    const { selection, service } = await mockService();

    await service.run("kaiser");

    setMockInvokeResponses({
      ["plugin:textures|describe"]: mockTextureDescription("ston\\ston_beton06"),
      ["plugin:textures|read_texture"]: new ArrayBuffer(0),
    });

    await selection.openFile("C:\\gamedata\\textures\\ston\\ston_beton06.dds");

    expect(service.comparison).toBeNull();
  });

  it("chooses a candidate and releases it when it is chosen again", async () => {
    const { service } = await mockService();

    await service.run("kaiser");
    service.choose("bc3");

    expect(service.isDirty).toBe(true);
    expect(service.chosenReport?.label).toBe("BC3 (DXT5)");

    service.choose("bc3");

    expect(service.isDirty).toBe(false);
    expect(service.chosenReport).toBeNull();
  });

  it("drops a chosen candidate when the formats are weighed again", async () => {
    // The backend replaces its held session whole, so a name kept from the previous comparison would address bytes
    // that are gone. Better to make the person pick again than to save something they did not look at.
    const { service } = await mockService();

    await service.run("kaiser");
    service.choose("bc1");
    await service.run("kaiser");

    expect(service.chosen).toBeNull();
    expect(service.isDirty).toBe(false);
  });

  it("forgets everything when cleared", async () => {
    const { service } = await mockService();

    await service.run("kaiser");
    service.choose("bc1");
    service.clear();

    expect(service.chosen).toBeNull();
    expect(service.comparison).toBeNull();
  });

  it("reads the chosen candidate as a picture, and lets it go again", async () => {
    // Choosing is what says somebody wants to look at the format rather than only read its numbers, so the picture is
    // fetched then rather than for every candidate a comparison weighed.
    const { service } = await mockService();

    await service.run(null);

    expect(service.preview.value).toBeNull();

    await service.choose("bc3");

    expect(service.chosen).toBe("bc3");
    expect(service.preview.value).toEqual(new ArrayBuffer(8));

    // Naming the held candidate again releases it, and the picture of it goes with the choice.
    await service.choose("bc3");

    expect(service.chosen).toBeNull();
    expect(service.preview.value).toBeNull();
  });

  it("answers the choice before it has the picture", async () => {
    // The row paints from `chosen`, so a click that only showed as chosen once the bytes arrived would look dropped.
    const { service } = await mockService();

    await service.run(null);

    const choosing = service.choose("bc1");

    expect(service.chosen).toBe("bc1");

    await choosing;
  });

  it("keeps no picture of a candidate that could not be decoded", async () => {
    const { service } = await mockService();

    await service.run(null);

    setMockInvokeResponses({
      ["plugin:textures|read_candidate"]: () => {
        throw new Error("The held comparison does not carry that format");
      },
    });

    await service.choose("bc3");

    // Still chosen: the encode is there to save even when this picture of it is not.
    expect(service.chosen).toBe("bc3");
    expect(service.preview.value).toBeNull();
    expect(service.preview.error?.message).toContain("does not carry that format");
  });
});
