// Auto-generated rust bindings. Do not edit it manually.

/** The mutually exclusive contracts an extern can expose. */
export type ExportContractDescriptor =
  | { kind: "callable"; parameters: Array<ExportParameterDescriptor>; returns: ExportReturnDescriptor }
  | { kind: "value"; typing: string };

/** One extern declaration projected for the application-facing exports project. */
export type ExportDescriptor = {
  name: string;
  description: string | null;
  source: ExportSourceDescriptor;
} & ExportContractDescriptor;

/** One callable parameter projected for the application-facing exports project. */
export type ExportParameterDescriptor = {
  name: string;
  typing: string;
  description: string | null;
  isOptional: boolean;
};

/** The return contract of a callable extern. */
export type ExportReturnDescriptor = {
  typing: string;
  description: string | null;
};

/** The source text that declares one extern. */
export type ExportSourceContent = {
  name: string;
  path: string;
  line: number;
  endLine: number;
  content: string;
};

/** Project-relative source location of an extern declaration. */
export type ExportSourceDescriptor = {
  path: string;
  line: number;
  column: number;
  /** Last line of the declaration, inclusive, so its body can be fetched without parsing again. */
  endLine: number;
};

/** Parsed externs and the project they came from. */
export type ExportsProject = {
  root: string;
  declarations: Array<ExportDescriptor>;
};
