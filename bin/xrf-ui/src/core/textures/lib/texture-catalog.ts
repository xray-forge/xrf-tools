import {
  TextureCatalog,
  TextureEntry,
  TextureMaterialSummary,
  TextureRole,
  TextureSource,
} from "@/core/bindings/types/xrf-app";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { Nullable, Optional } from "@/lib/types/general";

/**
 * What a texture turns out to be, once its listing and its descriptor have been joined.
 */
export enum ETextureBadge {
  /** Both halves of the declared pair resolved to the files the declaration names. */
  BUMPED = "bumped",
  /** The bump shader runs, but at least one half is the engine's dummy or absent entirely. */
  DEGRADED = "degraded",
  /** A detail texture is named and one of the two flags that switch it on is set. */
  DETAIL = "detail",
  /** The descriptor's texture type makes the engine skip it whole, bump declaration included. */
  ENGINE_SKIPPED = "engineSkipped",
  /** A `.thm` sits there and does not parse as one. */
  UNREADABLE = "unreadable",
  /** A descriptor with no texture beside it, which the engine still reads. */
  ORPHAN = "orphan",
  /** A file named as a bump half that no descriptor declares, so the engine never binds it. */
  UNREFERENCED = "unreferenced",
  /** The winning file is inside a `.db` volume, so it can be read and shown but not written. */
  ARCHIVED = "archived",
}

/** One texture of the browsed roots: what it is named, what files it has, and what its descriptor makes of them. */
export interface ITextureNode {
  /** What this row is called and keyed by: an engine reference, or a loose file's path below its root. */
  reference: string;
  /** Where its texture is, which is what opening the row asks for. */
  source: TextureSource;
  role: TextureRole;
  texture: Nullable<XrayAsset>;
  descriptor: Nullable<XrayAsset>;
  /** The pair halves this texture's own descriptor declares, taken out of the listing so they are not rows too. */
  halves: ReadonlyArray<TextureEntry>;
  /** What its own descriptor came to, absent for a texture that has none. */
  summary: Nullable<TextureMaterialSummary>;
  badges: ReadonlySet<ETextureBadge>;
}

/** How a badge reads, and how loudly. */
export interface ITextureBadgeDescriptor {
  label: string;
  /** Why the badge is worth a chip, for the tooltip and the filter's own title. */
  description: string;
  color: "default" | "success" | "info" | "warning" | "error";
}

const BADGE_DESCRIPTORS: Record<ETextureBadge, ITextureBadgeDescriptor> = {
  [ETextureBadge.BUMPED]: {
    color: "success",
    description: "Both halves of the declared bump pair are the files the descriptor names.",
    label: "Bumped",
  },
  [ETextureBadge.DEGRADED]: {
    color: "warning",
    description:
      "The engine takes the bump shader path but binds its flat dummy or nothing at all for a half, so the surface " +
      "renders flat while paying for the bump.",
    label: "Degraded",
  },
  [ETextureBadge.DETAIL]: {
    color: "info",
    description: "The descriptor names a detail texture and sets a flag that switches it on.",
    label: "Detail",
  },
  [ETextureBadge.ENGINE_SKIPPED]: {
    color: "warning",
    description:
      "The descriptor's texture type is one the engine skips whole, so its bump declaration is never read however " +
      "complete it looks.",
    label: "Skipped type",
  },
  [ETextureBadge.UNREADABLE]: {
    color: "error",
    description: "A descriptor file sits beside the texture and does not parse as one.",
    label: "Unreadable",
  },
  [ETextureBadge.ORPHAN]: {
    color: "info",
    description: "A descriptor with no texture beside it. The engine reads it either way.",
    label: "No texture",
  },
  [ETextureBadge.UNREFERENCED]: {
    color: "default",
    description: "A file named as a bump half that no descriptor declares, so the engine never binds it.",
    label: "Unreferenced",
  },
  [ETextureBadge.ARCHIVED]: {
    color: "default",
    description:
      "The file the roots answer with is inside a .db volume. It reads and shows like any other, and nothing can " +
      "write it where it is.",
    label: "Archived",
  },
};

/**
 * How a badge reads.
 *
 * @param badge - Badge to describe.
 * @returns Its label, its tooltip, and the severity its chip carries.
 */
export function describeTextureBadge(badge: ETextureBadge): ITextureBadgeDescriptor {
  return BADGE_DESCRIPTORS[badge];
}

/**
 * Joins the catalog listing to the descriptor sweep, folding each declared bump pair under the texture declaring it.
 *
 * @param entries - The catalog listing, one entry per engine reference.
 * @param summaries - What the sweep made of each descriptor, empty while it is still running.
 * @returns One node per texture, in the order the listing gave.
 */
export function buildTextureNodes(
  entries: ReadonlyArray<TextureEntry>,
  summaries: ReadonlyArray<TextureMaterialSummary>
): Array<ITextureNode> {
  const byReference: Map<string, TextureEntry> = new Map(
    entries.map((entry: TextureEntry) => [entry.reference, entry])
  );
  const summaryByReference: Map<string, TextureMaterialSummary> = new Map(
    summaries.map((summary: TextureMaterialSummary) => [summary.reference, summary])
  );

  // Both halves of every declared pair, mapped to whoever declared them. First declaration wins, so two textures
  // sharing one bump fold it under the earlier of them rather than under whichever was swept last.
  const declaredBy: Map<string, string> = new Map();

  for (const summary of summaries) {
    for (const half of [summary.bump?.bump, summary.bump?.companion]) {
      // A descriptor naming itself as its own bump would otherwise fold the texture out of its own tree.
      if (half && half !== summary.reference && !declaredBy.has(half)) {
        declaredBy.set(half, summary.reference);
      }
    }
  }

  const nodes: Array<ITextureNode> = [];

  for (const entry of entries) {
    if (declaredBy.has(entry.reference)) {
      continue;
    }

    const summary: Nullable<TextureMaterialSummary> = summaryByReference.get(entry.reference) ?? null;

    nodes.push({
      badges: collectBadges(entry, summary, declaredBy),
      descriptor: entry.descriptor,
      halves: collectHalves(summary, byReference),
      reference: entry.reference,
      role: entry.role,
      source: entry.source,
      summary,
      texture: entry.texture,
    });
  }

  return nodes;
}

/**
 * Narrows nodes to the badges a person asked for.
 *
 * A node matching any selected badge passes, rather than every one: the badges are conditions a texture is in, and
 * asking for degraded and unreadable together means "show me both kinds of trouble" rather than the empty set of
 * textures that are somehow both.
 *
 * @param nodes - Every node of the browsed roots.
 * @param badges - Badges selected in the header, empty for no filtering at all.
 * @returns The nodes to draw.
 */
export function filterTextureNodes(
  nodes: ReadonlyArray<ITextureNode>,
  badges: ReadonlySet<ETextureBadge>
): Array<ITextureNode> {
  if (!badges.size) {
    return [...nodes];
  }

  return nodes.filter((node: ITextureNode) => [...badges].some((badge: ETextureBadge) => node.badges.has(badge)));
}

/**
 * How many nodes each badge would select.
 *
 * Counted over every node rather than over the filtered ones, so a chip keeps saying how much it would show while
 * another chip is holding the tree down to a handful.
 *
 * @param nodes - Every node of the browsed roots.
 * @returns The count per badge, zero included.
 */
export function countTextureBadges(nodes: ReadonlyArray<ITextureNode>): Map<ETextureBadge, number> {
  const counts: Map<ETextureBadge, number> = new Map(
    Object.values(ETextureBadge).map((badge: ETextureBadge) => [badge, 0])
  );

  for (const node of nodes) {
    for (const badge of node.badges) {
      counts.set(badge, (counts.get(badge) ?? 0) + 1);
    }
  }

  return counts;
}

/**
 * Whether the roots hold a `textures.ltx`, whose declarations nothing here reads.
 *
 * @param catalog - The listing, or null when nothing is open.
 * @returns The file's engine path when the roots hold one, otherwise null.
 */
export function selectUnreadTexturesLtx(catalog: Nullable<TextureCatalog>): Nullable<string> {
  return catalog?.texturesLtx?.logicalPath ?? null;
}

/** The pair halves a descriptor declares, as far as the listing holds them. */
function collectHalves(
  summary: Nullable<TextureMaterialSummary>,
  byReference: ReadonlyMap<string, TextureEntry>
): Array<TextureEntry> {
  if (!summary?.bump) {
    return [];
  }

  return [summary.bump.bump, summary.bump.companion]
    .map((half: string) => byReference.get(half))
    .filter((entry: Optional<TextureEntry>): entry is TextureEntry => Boolean(entry));
}

/** What a node is, from its own descriptor and from where it stands in the listing. */
function collectBadges(
  entry: TextureEntry,
  summary: Nullable<TextureMaterialSummary>,
  declaredBy: ReadonlyMap<string, string>
): ReadonlySet<ETextureBadge> {
  const badges: Set<ETextureBadge> = new Set();

  if (summary?.badges.isBumped) {
    badges.add(ETextureBadge.BUMPED);
  }

  if (summary?.badges.isDegraded) {
    badges.add(ETextureBadge.DEGRADED);
  }

  if (summary?.badges.isDetailAssociated) {
    badges.add(ETextureBadge.DETAIL);
  }

  if (summary?.badges.isEngineSkipped) {
    badges.add(ETextureBadge.ENGINE_SKIPPED);
  }

  if (summary?.badges.isUnreadable) {
    badges.add(ETextureBadge.UNREADABLE);
  }

  if (!entry.texture && entry.descriptor) {
    badges.add(ETextureBadge.ORPHAN);
  }

  if (entry.role !== "texture" && !declaredBy.has(entry.reference)) {
    badges.add(ETextureBadge.UNREFERENCED);
  }

  if (entry.texture?.container.kind === "archive") {
    badges.add(ETextureBadge.ARCHIVED);
  }

  return badges;
}
