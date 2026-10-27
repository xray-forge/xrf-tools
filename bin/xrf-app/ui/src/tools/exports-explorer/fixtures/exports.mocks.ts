import { ExportDescriptor, ExportsProject } from "@xrf/ipc/types/xrf-export";

import { TCallableExportDescriptor } from "@/tools/exports-explorer/lib/export-descriptors";

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
