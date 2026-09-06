import {
  TextureBadges,
  TextureCatalog,
  TextureDescription,
  TextureEntry,
  TextureMaterialSummary,
  TextureRole,
  TextureVocabulary,
} from "@/core/bindings/types/xrf-app";
import { XrayMaterialDescriptor } from "@/core/bindings/types/xrf-material";
import { XrayAsset, XrayRoots } from "@/core/bindings/types/xrf-vfs";

/** The engine reference of a texture, its bump, and that bump's companion, which most cases need together. */
export const MOCK_TEXTURE: string = "ston\\ston_beton05";
export const MOCK_BUMP: string = "ston\\ston_beton05_bump";
export const MOCK_COMPANION: string = "ston\\ston_beton05_bump#";

/** A loose file of the browsed tree. */
export function mockTextureAsset(logicalPath: string, root: string = "C:\\gamedata"): XrayAsset {
  return { container: { kind: "directory", relativePath: logicalPath, root }, logicalPath };
}

/** An entry served out of an archive volume rather than a loose tree. */
export function mockArchivedTextureAsset(logicalPath: string): XrayAsset {
  return { container: { kind: "archive", path: "C:\\game\\db\\textures.db0" }, logicalPath };
}

export function mockTextureBadges(overrides: Partial<TextureBadges> = {}): TextureBadges {
  return {
    isBumped: false,
    isDegraded: false,
    isDetailAssociated: false,
    isEngineSkipped: false,
    isUnreadable: false,
    ...overrides,
  };
}

/**
 * One row of the catalog listing.
 *
 * Both files by default, since a texture with neither would not be listed at all; a case wanting an orphan descriptor
 * or a bare file overrides one of them with `null`.
 */
export function mockTextureEntry(reference: string, overrides: Partial<TextureEntry> = {}): TextureEntry {
  return {
    descriptor: mockTextureAsset(`textures\\${reference}.thm`),
    reference,
    role: roleOf(reference),
    texture: mockTextureAsset(`textures\\${reference}.dds`),
    ...overrides,
  };
}

/** What the sweep made of one descriptor. */
export function mockTextureSummary(
  reference: string,
  overrides: Partial<TextureMaterialSummary> = {}
): TextureMaterialSummary {
  return { badges: mockTextureBadges(), bump: null, reference, ...overrides };
}

/** A summary declaring a bump pair, which is what folds both halves under this texture. */
export function mockBumpedTextureSummary(
  reference: string = MOCK_TEXTURE,
  bump: string = MOCK_BUMP,
  companion: string = MOCK_COMPANION
): TextureMaterialSummary {
  return mockTextureSummary(reference, {
    badges: mockTextureBadges({ isBumped: true }),
    bump: { bump, companion },
  });
}

export function mockTextureRoots(root: string = "C:\\gamedata"): XrayRoots {
  return { asset: null, roots: [{ mode: "auto", path: root }] };
}

export function mockTextureCatalog(
  entries: Array<TextureEntry>,
  overrides: Partial<TextureCatalog> = {}
): TextureCatalog {
  return { entries, outsideTexturesCount: 0, roots: mockTextureRoots(), texturesLtx: null, ...overrides };
}

/** A material the engine reads nothing from, which is what most textures have. */
export function mockFlatMaterial(overrides: Partial<XrayMaterialDescriptor> = {}): XrayMaterialDescriptor {
  return {
    bump: null,
    declaration: { kind: "noDescriptor" },
    descriptor: null,
    detail: null,
    outcome: "flat",
    ...overrides,
  };
}

export function mockTextureDescription(
  reference: string = MOCK_TEXTURE,
  overrides: Partial<TextureDescription> = {}
): TextureDescription {
  return {
    base: { shape: null, size: 1024 },
    bump: null,
    companion: null,
    form: null,
    material: mockFlatMaterial(),
    reference,
    roots: mockTextureRoots(),
    source: { kind: "asset", reference },
    targets: null,
    texture: mockTextureAsset(`textures\\${reference}.dds`),
    ...overrides,
  };
}

/** The role the backend reads off a name, mirrored here so a fixture does not have to state the obvious. */
function roleOf(reference: string): TextureRole {
  if (reference.endsWith("#")) {
    return "bumpCompanion";
  }

  return reference.endsWith("_bump") ? "bump" : "texture";
}

/**
 * The names the SDK gives a descriptor's numbers, as the backend answers them.
 *
 * Short rather than complete: a test asserting how a field is named needs one entry it can point at, and a full table
 * would make every one of them a place to remember when the format gains a value.
 *
 * @param overrides - Fields to replace.
 * @returns A vocabulary a form can render.
 */
export function mockTextureVocabulary(overrides: Partial<TextureVocabulary> = {}): TextureVocabulary {
  return {
    bumpModes: [
      { label: "None", value: 1 },
      { label: "Use", value: 2 },
    ],
    bumpModeUse: 2,
    flags: [
      { bit: 1, label: "flGenerateMipMaps" },
      { bit: 1 << 25, label: "flHasAlpha" },
    ],
    formats: [
      { label: "tfDXT1", value: 0 },
      { label: "tfDXT5", value: 4 },
    ],
    materials: [{ label: "mtOrenNayar_Blin", value: 0 }],
    mipFilters: [{ label: "kMIPFilterBox", value: 1 }],
    textureTypes: [
      { label: "ttImage", value: 0 },
      { label: "ttTerrain", value: 4 },
    ],
    ...overrides,
  };
}
