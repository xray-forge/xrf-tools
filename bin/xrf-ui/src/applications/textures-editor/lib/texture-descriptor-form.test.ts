import { describe, expect, it } from "@jest/globals";

import { mockTextureDescriptorForm as form, mockTextureDescription } from "@/fixtures/mocks/texture.mocks";

import { isSameDescriptorForm, toEditableForm } from "./texture-descriptor-form";

describe("toEditableForm", () => {
  it("should offer the backend's authoring form for a texture with no descriptor, because most textures have none", () => {
    const authoring = form({ flags: 3, material: 1 });
    const editable = toEditableForm(
      mockTextureDescription("ston\\ston_beton05", {
        base: { shape: null, size: 1 },
        form: authoring,
        hasDescriptor: false,
      })
    );

    expect(editable).toEqual(authoring);
  });

  it("should take width and height from the dds header rather than from the descriptor's own copy", () => {
    // The descriptor's copy is authoring data a converter refreshed; a hand-edited file can have it wrong, and the
    // header is what the engine actually uploads.
    const editable = toEditableForm(
      mockTextureDescription("ston\\ston_beton05", {
        base: { shape: { format: "DXT5", height: 512, mipmapLevels: 10, width: 256 }, size: 1024 },
        form: form({ height: 4, width: 4 }),
      })
    );

    expect(editable?.width).toBe(256);
    expect(editable?.height).toBe(512);
  });

  it("should keep the descriptor's own size when the header could not be read", () => {
    const editable = toEditableForm(
      mockTextureDescription("ston\\ston_beton05", { base: { shape: null, size: 1024 }, form: form({ width: 64 }) })
    );

    expect(editable?.width).toBe(64);
  });

  it("should answer null for nothing selected", () => {
    expect(toEditableForm(null)).toBeNull();
  });
});

describe("isSameDescriptorForm", () => {
  it("should treat a field edited back to what it was as no change at all", () => {
    // Otherwise a person who types over a name and types it again is prompted to save a file that is already what
    // they want.
    expect(isSameDescriptorForm(form({ bumpName: "a" }), form({ bumpName: "a" }))).toBe(true);
    expect(isSameDescriptorForm(form({ bumpName: "a" }), form({ bumpName: "b" }))).toBe(false);
  });

  it("should compare nulls as equal and a null against a form as different", () => {
    expect(isSameDescriptorForm(null, null)).toBe(true);
    expect(isSameDescriptorForm(form(), null)).toBe(false);
  });
});
