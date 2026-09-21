import { describe, expect, it } from "@jest/globals";

import { ArchiveReadPolicy } from "@/core/ipc/types/xrf-archive";
import { mockArchiveFileDescriptor, mockArchiveReadPolicy } from "@/fixtures/mocks/archive.mocks";

import { getArchivePreviewSupport } from "./preview";

const READ_POLICY: ArchiveReadPolicy = mockArchiveReadPolicy();

describe("archive preview support", () => {
  it.each(READ_POLICY.extensions)("accepts uncompressed .%s files within the backend limit", (extension: string) => {
    expect(getArchivePreviewSupport(mockArchiveFileDescriptor({ name: `preview.${extension}` }), READ_POLICY)).toEqual({
      kind: "supported",
    });
  });

  it("accepts the normalized extension regardless of filename casing", () => {
    expect(getArchivePreviewSupport(mockArchiveFileDescriptor({ name: "actor.SCRIPT" }), READ_POLICY)).toEqual({
      kind: "supported",
    });
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

  it("draws source art itself rather than sending it through the texture decode", () => {
    // These are not textures and the engine loads none of them. Routing one as an image would hand a png to a DDS
    // header parse; routing it as a description would say nothing about a file a webview can simply draw.
    for (const name of ["textures\\ui\\source.png", "textures\\ui\\source.JPG", "textures\\ui\\source.bmp"]) {
      expect(getArchivePreviewSupport(mockArchiveFileDescriptor({ name }), READ_POLICY)).toEqual({
        kind: "image",
      });
    }
  });

  it("keeps the texture decode for the one picture format the engine actually loads", () => {
    expect(getArchivePreviewSupport(mockArchiveFileDescriptor({ name: "textures\\wall.dds" }), READ_POLICY)).toEqual({
      kind: "texture",
    });
  });

  it("does not draw a targa itself, because no webview renders one", () => {
    // It stays a description rather than a picture: drawing it would need the transcode the dds path takes.
    expect(getArchivePreviewSupport(mockArchiveFileDescriptor({ name: "textures\\source.tga" }), READ_POLICY)).toEqual({
      kind: "description",
    });
  });

  it("plays a wav beside an ogg, since both are handed to the webview as they stand", () => {
    expect(getArchivePreviewSupport(mockArchiveFileDescriptor({ name: "sounds\\source.wav" }), READ_POLICY)).toEqual({
      kind: "audio",
    });
  });

  it("refuses a picture past the native limit rather than holding it whole", () => {
    expect(
      getArchivePreviewSupport(
        mockArchiveFileDescriptor({
          name: "textures\\ui\\source.png",
          sizeReal: READ_POLICY.maximumImageSize + 1,
        }),
        READ_POLICY
      )
    ).toEqual({ kind: "too-large", maximumSize: READ_POLICY.maximumImageSize });
  });

  it("sends an unreadable extension to the backend and refuses an oversized text read", () => {
    expect(getArchivePreviewSupport(mockArchiveFileDescriptor({ name: "meshes\\actor.omf" }), READ_POLICY)).toEqual({
      kind: "description",
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
    ).toEqual({ kind: "texture" });

    expect(
      getArchivePreviewSupport(
        mockArchiveFileDescriptor({ name: "textures\\ui.dds", sizeReal: READ_POLICY.maximumTextureSize + 1 }),
        READ_POLICY
      )
    ).toEqual({ kind: "too-large", maximumSize: READ_POLICY.maximumTextureSize });
  });

  it("reads the extension from the entry name rather than from a directory above it", () => {
    expect(
      getArchivePreviewSupport(mockArchiveFileDescriptor({ name: "configs\\v1.5\\system.ltx" }), READ_POLICY)
    ).toEqual({ kind: "supported" });
  });

  it("accepts a sound exactly at the backend's audio limit", () => {
    const policy: ArchiveReadPolicy = mockArchiveReadPolicy({ maximumAudioSize: 4096 });
    const sound = mockArchiveFileDescriptor({ name: "sounds\\wind.OGG", sizeReal: 4096 });

    expect(getArchivePreviewSupport(sound, policy)).toEqual({ kind: "audio" });
  });

  it("reports the audio limit when a sound exceeds it", () => {
    const policy: ArchiveReadPolicy = mockArchiveReadPolicy({ maximumAudioSize: 4096 });
    const sound = mockArchiveFileDescriptor({ name: "sounds\\wind.ogg", sizeReal: 4097 });

    expect(getArchivePreviewSupport(sound, policy)).toEqual({ kind: "too-large", maximumSize: 4096 });
  });

  it("uses backend-provided policy values", () => {
    const policy: ArchiveReadPolicy = mockArchiveReadPolicy({
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
      kind: "description",
    });
  });
});
