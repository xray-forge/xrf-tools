import { ExportDescriptor } from "@/core/ipc/types/xrf-export";

/**
 * The two halves of the generated descriptor union, for call sites that have already narrowed.
 *
 * Derived rather than declared: the backend models the contract as one struct with a flattened enum, so
 * these follow whatever that becomes instead of restating it.
 */
export type TCallableExportDescriptor = Extract<ExportDescriptor, { kind: "callable" }>;

export type TValueExportDescriptor = Extract<ExportDescriptor, { kind: "value" }>;
