import { describe, expect, it } from "@jest/globals";

import { TextureEntry, TextureMaterialSummary } from "@/core/bindings/types/xrf-app";
import {
  buildTextureNodes,
  countTextureBadges,
  ETextureBadge,
  filterTextureNodes,
  ITextureNode,
} from "@/core/textures/lib/texture-catalog";
import {
  MOCK_BUMP,
  MOCK_COMPANION,
  MOCK_TEXTURE,
  mockArchivedTextureAsset,
  mockBumpedTextureSummary,
  mockTextureBadges,
  mockTextureEntry,
  mockTextureSummary,
} from "@/fixtures/mocks/texture.mocks";

/** The listing a declared pair produces: the texture and both of its halves. */
function mockPairEntries(): Array<TextureEntry> {
  return [mockTextureEntry(MOCK_TEXTURE), mockTextureEntry(MOCK_BUMP), mockTextureEntry(MOCK_COMPANION)];
}

function referencesOf(nodes: Array<ITextureNode>): Array<string> {
  return nodes.map((node: ITextureNode) => node.reference);
}

describe("buildTextureNodes", () => {
  it("lists every texture when no descriptor declares anything", () => {
    const nodes: Array<ITextureNode> = buildTextureNodes(mockPairEntries(), []);

    // Nothing has said the halves belong to anything yet, so they stand on their own.
    expect(referencesOf(nodes)).toEqual([MOCK_TEXTURE, MOCK_BUMP, MOCK_COMPANION]);
  });

  it("folds a declared pair under the texture declaring it", () => {
    const nodes: Array<ITextureNode> = buildTextureNodes(mockPairEntries(), [mockBumpedTextureSummary()]);

    expect(referencesOf(nodes)).toEqual([MOCK_TEXTURE]);
    expect(nodes[0].halves.map((half: TextureEntry) => half.reference)).toEqual([MOCK_BUMP, MOCK_COMPANION]);
    expect(nodes[0].badges.has(ETextureBadge.BUMPED)).toBe(true);
  });

  it("keeps a half no descriptor declares, and marks it", () => {
    const nodes: Array<ITextureNode> = buildTextureNodes(
      [mockTextureEntry(MOCK_TEXTURE), mockTextureEntry("wpn\\wpn_ak74_bump")],
      [mockBumpedTextureSummary()]
    );

    // The declared pair folds away; a bump belonging to nothing is what the modder is looking for.
    expect(referencesOf(nodes)).toEqual([MOCK_TEXTURE, "wpn\\wpn_ak74_bump"]);
    expect(nodes[1].badges.has(ETextureBadge.UNREFERENCED)).toBe(true);
  });

  it("marks a descriptor with no texture beside it", () => {
    const nodes: Array<ITextureNode> = buildTextureNodes(
      [mockTextureEntry(MOCK_TEXTURE, { texture: null })],
      [mockTextureSummary(MOCK_TEXTURE)]
    );

    expect(nodes[0].badges.has(ETextureBadge.ORPHAN)).toBe(true);
  });

  it("carries every badge the sweep reported", () => {
    const summary: TextureMaterialSummary = mockTextureSummary(MOCK_TEXTURE, {
      badges: mockTextureBadges({ isDegraded: true, isDetailAssociated: true, isEngineSkipped: true }),
    });
    const nodes: Array<ITextureNode> = buildTextureNodes([mockTextureEntry(MOCK_TEXTURE)], [summary]);

    expect([...nodes[0].badges].sort()).toEqual(
      [ETextureBadge.DEGRADED, ETextureBadge.DETAIL, ETextureBadge.ENGINE_SKIPPED].sort()
    );
  });

  it("keeps a texture that declares itself as its own bump", () => {
    // Pathological authoring, but folding it would take the texture out of its own tree and leave nothing to fix it
    // with.
    const nodes: Array<ITextureNode> = buildTextureNodes(
      [mockTextureEntry(MOCK_TEXTURE)],
      [mockBumpedTextureSummary(MOCK_TEXTURE, MOCK_TEXTURE, MOCK_COMPANION)]
    );

    expect(referencesOf(nodes)).toEqual([MOCK_TEXTURE]);
  });

  it("folds a half under the first texture declaring it when two do", () => {
    const nodes: Array<ITextureNode> = buildTextureNodes(
      [mockTextureEntry(MOCK_TEXTURE), mockTextureEntry("ston\\ston_beton06"), mockTextureEntry(MOCK_BUMP)],
      [mockBumpedTextureSummary(), mockBumpedTextureSummary("ston\\ston_beton06")]
    );

    expect(referencesOf(nodes)).toEqual([MOCK_TEXTURE, "ston\\ston_beton06"]);
    expect(nodes[0].halves).toHaveLength(1);
    expect(nodes[1].halves).toHaveLength(1);
  });

  it("marks a texture served out of an archive rather than hiding or disabling it", () => {
    // Decision 20 as it survives decision 24. The only tree left is the read-only one, where an archived texture is
    // fully inspectable, so the row says what a person can act on: this file cannot be written where it is.
    const nodes: Array<ITextureNode> = buildTextureNodes(
      [mockTextureEntry(MOCK_TEXTURE, { texture: mockArchivedTextureAsset(`textures\\${MOCK_TEXTURE}.dds`) })],
      []
    );

    expect(referencesOf(nodes)).toEqual([MOCK_TEXTURE]);
    expect(nodes[0].badges.has(ETextureBadge.ARCHIVED)).toBe(true);
  });

  it("carries the address a row is opened by, which is not always its label", () => {
    // A loose listing labels rows by path and opens them by file; a game tree labels and opens by reference. The row
    // is the only thing that knows which of the two it is.
    const nodes: Array<ITextureNode> = buildTextureNodes(
      [
        mockTextureEntry(MOCK_TEXTURE),
        mockTextureEntry("brick01", { source: { kind: "file", path: "C:\\work\\brick01.dds" } }),
      ],
      []
    );

    expect(nodes[0].source).toEqual({ kind: "asset", reference: MOCK_TEXTURE });
    expect(nodes[1].source).toEqual({ kind: "file", path: "C:\\work\\brick01.dds" });
  });
});

describe("filterTextureNodes", () => {
  /** One texture per badge worth filtering by, so a selection can be checked against a known set. */
  function mockMixedNodes(): Array<ITextureNode> {
    return buildTextureNodes(
      [mockTextureEntry("a\\bumped"), mockTextureEntry("a\\degraded"), mockTextureEntry("a\\plain")],
      [
        mockTextureSummary("a\\bumped", { badges: mockTextureBadges({ isBumped: true }) }),
        mockTextureSummary("a\\degraded", { badges: mockTextureBadges({ isDegraded: true }) }),
        mockTextureSummary("a\\plain"),
      ]
    );
  }

  it("returns everything when nothing is selected", () => {
    expect(filterTextureNodes(mockMixedNodes(), new Set())).toHaveLength(3);
  });

  it("selects a node matching any chosen badge rather than every one", () => {
    const filtered: Array<ITextureNode> = filterTextureNodes(
      mockMixedNodes(),
      new Set([ETextureBadge.BUMPED, ETextureBadge.DEGRADED])
    );

    expect(referencesOf(filtered)).toEqual(["a\\bumped", "a\\degraded"]);
  });
});

describe("countTextureBadges", () => {
  it("counts every badge, including the ones nothing carries", () => {
    const counts: Map<ETextureBadge, number> = countTextureBadges(
      buildTextureNodes(
        [mockTextureEntry("a\\one"), mockTextureEntry("a\\two")],
        [
          mockTextureSummary("a\\one", { badges: mockTextureBadges({ isBumped: true }) }),
          mockTextureSummary("a\\two", { badges: mockTextureBadges({ isBumped: true, isDegraded: true }) }),
        ]
      )
    );

    expect(counts.get(ETextureBadge.BUMPED)).toBe(2);
    expect(counts.get(ETextureBadge.DEGRADED)).toBe(1);
    expect(counts.get(ETextureBadge.UNREADABLE)).toBe(0);
  });
});
