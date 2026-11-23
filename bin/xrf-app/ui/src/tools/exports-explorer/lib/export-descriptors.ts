import { ExportDescriptor } from "@xrf/ipc/types/xrf-export";

/**
 * The callable half of the generated descriptor union, for call sites that have already narrowed.
 *
 * Derived rather than declared: the backend models the contract as one struct with a flattened enum, so
 * this follows whatever that becomes instead of restating it.
 */
export type TCallableExportDescriptor = Extract<ExportDescriptor, { kind: "callable" }>;
