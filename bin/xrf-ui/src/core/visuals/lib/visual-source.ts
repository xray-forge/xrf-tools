import { VisualSource } from "@/core/ipc/types/xrf-app";

/**
 * Human readable name of where a visual came from.
 *
 * Switched on `kind` rather than reading a field one variant happens to have, so adding a source is a compile error
 * here instead of silently reading `undefined` — which is exactly what caught the asset variant.
 *
 * An asset is named by its engine identity, which is what a browsed tree shows and what the user recognizes; it may
 * live inside a volume and have no filesystem path to print.
 *
 * @param source - The visual source to describe.
 * @returns A label identifying the source.
 */
export function describeVisualSource(source: VisualSource): string {
  switch (source.kind) {
    case "file":
      return source.path;

    case "asset":
      return source.logicalPath;
  }
}
