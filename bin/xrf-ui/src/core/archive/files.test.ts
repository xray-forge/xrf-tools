import { describe, expect, it } from "@jest/globals";

import { findSharedPayloadOf, listPayloadSharersOf } from "@/core/archive/files";
import { ArchiveFileDescriptor, ArchiveSharedPayload } from "@/core/bindings/types/xrf-archive";
import { mockArchiveFileDescriptor, mockArchiveSharedPayload } from "@/fixtures/mocks/archive.mocks";

const FIRST: ArchiveFileDescriptor = mockArchiveFileDescriptor({ name: "configs\\first.ltx", offset: 64 });
const SECOND: ArchiveFileDescriptor = mockArchiveFileDescriptor({ name: "configs\\second.ltx", offset: 64 });
const OWN: ArchiveFileDescriptor = mockArchiveFileDescriptor({ name: "configs\\own.ltx", offset: 4096 });

const SHARED: Array<ArchiveSharedPayload> = [mockArchiveSharedPayload(FIRST, [SECOND.name])];

describe("archive shared payloads", () => {
  it("finds an entry's payload by what it reads rather than by its name", () => {
    // Both rows carry the same location, so either name finds the one group; a row elsewhere finds nothing.
    expect(findSharedPayloadOf(SHARED, FIRST)).toBe(SHARED[0]);
    expect(findSharedPayloadOf(SHARED, SECOND)).toBe(SHARED[0]);
    expect(findSharedPayloadOf(SHARED, OWN)).toBeNull();
    expect(findSharedPayloadOf(SHARED, null)).toBeNull();
  });

  it("lists the other names and never the entry's own", () => {
    expect(listPayloadSharersOf(SHARED, FIRST)).toEqual(["configs\\second.ltx"]);
    expect(listPayloadSharersOf(SHARED, SECOND)).toEqual(["configs\\first.ltx"]);
    expect(listPayloadSharersOf(SHARED, OWN)).toEqual([]);
  });

  it("treats a directory row as sharing nothing", () => {
    // A directory row has no payload; matching it on zero fields would group every directory in a volume.
    const directory: ArchiveFileDescriptor = mockArchiveFileDescriptor({ name: "configs\\", sizeReal: 0 });

    expect(findSharedPayloadOf([mockArchiveSharedPayload(directory, ["textures\\"])], directory)).toBeNull();
  });
});
