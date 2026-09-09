import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container } from "@wirestate/core";
import { isComputedProp } from "@wirestate/mobx";

import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { JobsService } from "@/core/jobs/services/jobs";
import { TextureSelectionService } from "@/core/textures/services/selection";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { MOCK_TEXTURE, mockTextureDescription, mockTextureEncodingComparison } from "@/fixtures/mocks/texture.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

import { TextureEncodingService } from "./texture-encoding.service";

function pendingPreview() {
  let resolve: (bytes: ArrayBuffer) => void = noop;
  const promise: Promise<ArrayBuffer> = new Promise((settle) => {
    resolve = settle;
  });

  return { promise, resolve };
}

async function mockService(): Promise<{ service: TextureEncodingService; selection: TextureSelectionService }> {
  setMockInvokeResponses({
    ["plugin:textures|compare_encodings"]: mockTextureEncodingComparison(),
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

    expect(isComputedProp(service, "chosen")).toBe(true);
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
  it("addresses candidate previews by the returned comparison session", async () => {
    const { service } = await mockService();

    await service.run(null);
    await service.choose("bc3");

    expect(mockInvoke).toHaveBeenLastCalledWith("plugin:textures|read_candidate", {
      sessionId: mockTextureEncodingComparison().sessionId,
      format: "bc3",
    });
  });

  it.each(["roots", "source"] as const)("does not reuse a same-label comparison with different %s", async (field) => {
    const { service, selection } = await mockService();

    await service.run(null);
    await service.choose("bc3");

    const description: TextureDescription = mockTextureDescription(MOCK_TEXTURE, {
      ...(field === "roots"
        ? { roots: { asset: null, roots: [{ path: "D:/other" }] } }
        : { source: { kind: "file", path: "D:/other/ston_beton05.dds" } }),
    });

    setMockInvokeResponses({
      ["plugin:textures|describe"]: description,
      ["plugin:textures|read_texture"]: new ArrayBuffer(0),
    });
    await selection.open(description.source, description.roots);

    expect(service.comparison).toBeNull();
    expect(service.chosen).toBeNull();
    expect(service.isDirty).toBe(false);
  });

  it("does not publish a pending preview after clearing its comparison", async () => {
    const { service } = await mockService();
    const pending = pendingPreview();

    await service.run(null);
    setMockInvokeResponses({ ["plugin:textures|read_candidate"]: () => pending.promise });

    const choosing = service.choose("bc3");

    service.clear();
    pending.resolve(new ArrayBuffer(8));
    await choosing;

    expect(service.preview.value).toBeNull();
    expect(service.chosen).toBeNull();
  });

  it("does not publish an old preview into a new comparison", async () => {
    const { service } = await mockService();
    const pending = pendingPreview();

    await service.run(null);
    setMockInvokeResponses({
      ["plugin:textures|read_candidate"]: () => pending.promise,
      ["plugin:textures|compare_encodings"]: mockTextureEncodingComparison({
        sessionId: "5ffb9fb7-b48b-4b55-b713-d7941d623cbe",
      }),
    });

    const choosing = service.choose("bc3");

    await service.run(null);
    pending.resolve(new ArrayBuffer(8));
    await choosing;

    expect(service.preview.value).toBeNull();
    expect(service.chosen).toBeNull();
  });
});
