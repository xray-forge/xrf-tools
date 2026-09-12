import { IChoiceFormRowOption } from "@/core/ui/form";

/** Which rules resolve the tree, named rather than flagged: the two read the same files to different values. */
export const enum EConfigsDialect {
  LTX = "ltx",
  DLTX = "dltx",
}

export const DIALECT_IDS: ReadonlyArray<EConfigsDialect> = [EConfigsDialect.LTX, EConfigsDialect.DLTX];

export const DIALECT_OPTIONS: ReadonlyArray<IChoiceFormRowOption<EConfigsDialect>> = [
  { "aria-label": "Standard LTX", value: EConfigsDialect.LTX, label: "Standard" },
  { "aria-label": "Monolith DLTX", value: EConfigsDialect.DLTX, label: "DLTX" },
];
