import { ArchiveDescriptor, ArchiveFileDescriptor, ArchiveProject } from "@/core/bindings/types/xrf-archive";
import { ExportDescriptor, ExportsProject } from "@/core/bindings/types/xrf-export";
import { TranslationProjectDescriptor, TranslationSource } from "@/core/bindings/types/xrf-translation";
import { TCallableExportDescriptor } from "@/core/exports";
import { IEquipmentSectionDescriptor } from "@/core/sprite-equipment";

import { mockArchiveFileDescriptor, mockArchiveReadPolicy } from "./archive.mocks";

/**
 * Creates an archive file fixture.
 *
 * @param overrides - Field values to override.
 * @returns An archive file fixture.
 */
export function mockArchiveFile(overrides: Partial<ArchiveFileDescriptor> = {}): ArchiveFileDescriptor {
  // Through the descriptor fixture, so `isDirectory` is derived in one place rather than spelled out in two.
  return mockArchiveFileDescriptor({
    crc: 123456,
    destination: "gamedata\\config\\system.ltx",
    extension: "ltx",
    name: "system.ltx",
    offset: 0,
    sizeCompressed: 512,
    sizeReal: 2048,
    source: "db\\db0",
    ...overrides,
  });
}

/**
 * Creates an archive descriptor fixture.
 *
 * @param overrides - Field values to override.
 * @returns An archive descriptor fixture.
 */
export function mockArchiveDescriptor(overrides: Partial<ArchiveDescriptor> = {}): ArchiveDescriptor {
  return {
    createdAt: null,
    modifiedAt: null,
    files: {},
    outputRootPath: "unpacked",
    path: "db\\db0",
    ...overrides,
  };
}

/**
 * Creates an archives project fixture.
 *
 * @param overrides - Field values to override.
 * @returns An archives project fixture.
 */
export function mockArchivesProject(overrides: Partial<ArchiveProject> = {}): ArchiveProject {
  return {
    archives: [mockArchiveDescriptor(), mockArchiveDescriptor({ path: "db\\db1" })],
    files: {
      "config\\system.ltx": mockArchiveFile(),
      "config\\weapons\\wpn_ak74.ltx": mockArchiveFile({
        name: "wpn_ak74.ltx",
        destination: "gamedata\\config\\weapons\\wpn_ak74.ltx",
      }),
      "meshes\\dynamics\\weapons\\wpn_ak74.ogf": mockArchiveFile({
        extension: "ogf",
        name: "wpn_ak74.ogf",
        destination: "gamedata\\meshes\\dynamics\\weapons\\wpn_ak74.ogf",
      }),
    },
    readPolicy: mockArchiveReadPolicy(),
    root: "db",
    sizeReal: 6144,
    ...overrides,
  };
}

/**
 * Creates a callable export fixture.
 *
 * @param overrides - Field values to override.
 * @returns A callable export fixture.
 */
export function mockExportDescriptor(overrides: Partial<TCallableExportDescriptor> = {}): TCallableExportDescriptor {
  return {
    kind: "callable",
    name: "play_sound",
    description: null,
    parameters: [{ name: "actor", typing: "game_object", description: null, isOptional: false }],
    returns: { typing: "void", description: null },
    source: { path: "xr_effects.ts", line: 42, column: 2, endLine: 45 },
    ...overrides,
  };
}

/**
 * Creates export declaration fixtures.
 *
 * @param overrides - Additional declarations to append.
 * @returns Export declaration fixtures followed by the additional declarations.
 */
export function mockExportsDeclarations(overrides: Array<ExportDescriptor> = []): Array<ExportDescriptor> {
  return [
    mockExportDescriptor({
      source: { path: "xr_conditions.ts", line: 1, column: 1, endLine: 4 },
      name: "xr_conditions.is_wounded",
    }),
    mockExportDescriptor({ source: { path: "dialogs.ts", line: 1, column: 1, endLine: 4 }, name: "dialogs.is_friend" }),
    mockExportDescriptor({ source: { path: "dialogs.ts", line: 1, column: 1, endLine: 4 }, name: "dialogs.has_item" }),
    mockExportDescriptor({ name: "xr_effects.play_sound" }),
    ...overrides,
  ];
}

/**
 * Creates an exports project fixture.
 *
 * @param overrides - Field values to override.
 * @returns An exports project fixture.
 */
export function mockExportsProject(overrides: Partial<ExportsProject> = {}): ExportsProject {
  return {
    root: "C:\\projects\\xrf",
    declarations: mockExportsDeclarations(),
    ...overrides,
  };
}

/**
 * Creates a translations project fixture.
 *
 * @param overrides - Field values to override.
 * @returns A translations project fixture.
 */
export function mockTranslationsProject(
  overrides: Partial<TranslationProjectDescriptor> = {}
): TranslationProjectDescriptor {
  const root: string = "C:\\projects\\xrf\\src\\engine";

  function source(name: string): TranslationSource {
    return {
      logicalPath: `translations\\${name}`,
      physicalPath: `${root}/translations/${name}`,
    };
  }

  return {
    mode: "source",
    roots: { asset: null, roots: [{ mode: "auto", path: root }] },
    prefix: "translations",
    languages: ["eng", "ukr"],
    encodings: { eng: "windows-1252", ukr: "windows-1251" },
    isEditable: true,
    files: {
      "st_dialogs.json": {
        sources: { eng: source("st_dialogs.json"), ukr: source("st_dialogs.json") },
        entries: {
          dialog_greeting: { eng: "Hello, stalker", ukr: "Pryvit, stalker" },
        },
      },
      "st_items.json": {
        sources: { eng: source("st_items.json"), ukr: source("st_items.json") },
        entries: {
          wpn_ak74: { eng: "AK-74", ukr: "AK-74" },
          wpn_ak74_descr: { eng: "Assault rifle", ukr: null },
        },
      },
    },
    findings: [],
    ...overrides,
  };
}

/**
 * Creates equipment descriptor fixtures.
 *
 * @returns Equipment descriptor fixtures.
 */
export function mockEquipmentDescriptors(): Array<IEquipmentSectionDescriptor> {
  return [
    { section: "wpn_ak74", w: 2, h: 1, x: 0, y: 0 },
    { section: "wpn_pm", w: 1, h: 1, x: 2, y: 0 },
  ];
}
