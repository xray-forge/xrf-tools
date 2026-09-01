import { describe, expect, it } from "@jest/globals";

import { getArchivePreviewSupport } from "@/core/archive/preview";
import { ArchiveProjectReadPolicy } from "@/core/bindings/types/xrf-archive";
import { mockArchiveFileDescriptor, mockArchiveReadPolicy } from "@/fixtures/mocks/archive.mocks";

const READ_POLICY: ArchiveProjectReadPolicy = mockArchiveReadPolicy();

describe("archive preview support", () => {
  it.each(READ_POLICY.extensions)("accepts uncompressed .%s files within the backend limit", (extension: string) => {
    expect(
      getArchivePreviewSupport(mockArchiveFileDescriptor({ name: `preview.${extension}` }), READ_POLICY)
    ).toEqual({ kind: "supported" });
  });

  it("accepts the normalized extension regardless of filename casing", () => {
    expect(
      getArchivePreviewSupport(mockArchiveFileDescriptor({ name: "actor.SCRIPT" }), READ_POLICY)
    ).toEqual({ kind: "supported" });
  });

  it("offers a model preview for a visual, which no policy limit gates", () => {
    // A model is never read through the archive project: it is addressed logically and read through the asset roots, so
    // the size ceilings that bound a text or image read do not apply to it.
    expect(
      getArchivePreviewSupport(
        mockArchiveFileDescriptor({
          name: "meshes\\actor.ogf",
          sizeReal: READ_POLICY.maximumSize + 1,
        }),
        READ_POLICY
      )
    ).toEqual({ kind: "model" });
  });

  it("identifies each unsupported reason before a backend read", () => {
    expect(
      getArchivePreviewSupport(mockArchiveFileDescriptor({ name: "meshes\\actor.omf" }), READ_POLICY)
    ).toEqual({
      kind: "unsupported-extension",
      extension: "omf",
    });
    expect(
      getArchivePreviewSupport(mockArchiveFileDescriptor({ sizeReal: 2048, sizeCompressed: 1024 }), READ_POLICY)
    ).toEqual({ kind: "supported" });
    expect(
      getArchivePreviewSupport(mockArchiveFileDescriptor({ sizeReal: READ_POLICY.maximumSize + 1 }), READ_POLICY)
    ).toEqual({ kind: "too-large", maximumSize: READ_POLICY.maximumSize });
  });

  it("routes textures to the image path, compressed or not", () => {
    expect(
      getArchivePreviewSupport(
        mockArchiveFileDescriptor({ name: "textures\\ui.dds", sizeReal: 2048, sizeCompressed: 512 }),
        READ_POLICY
      )
    ).toEqual({ kind: "image" });

    expect(
      getArchivePreviewSupport(
        mockArchiveFileDescriptor({ name: "textures\\ui.dds", sizeReal: READ_POLICY.maximumImageSize + 1 }),
        READ_POLICY
      )
    ).toEqual({ kind: "too-large", maximumSize: READ_POLICY.maximumImageSize });
  });

  it("reads the extension from the entry name rather than from a directory above it", () => {
    expect(
      getArchivePreviewSupport(mockArchiveFileDescriptor({ name: "configs\\v1.5\\system.ltx" }), READ_POLICY)
    ).toEqual({ kind: "supported" });
  });

  it("uses backend-provided policy values", () => {
    const policy: ArchiveProjectReadPolicy = mockArchiveReadPolicy({
      extensions: ["xml"],
      maximumSize: 1024,
    });

    expect(
      getArchivePreviewSupport(
        mockArchiveFileDescriptor({ name: "preview.xml", sizeReal: 1024, sizeCompressed: 512 }),
        policy
      )
    ).toEqual({ kind: "supported" });
    expect(getArchivePreviewSupport(mockArchiveFileDescriptor(), policy)).toEqual({
      kind: "unsupported-extension",
      extension: "ltx",
    });
  });
});
