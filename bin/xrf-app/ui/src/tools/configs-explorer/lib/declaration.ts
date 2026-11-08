import { LtxDeclaration } from "@xrf/ipc/types/xrf-ltx-inspect";

/**
 * One found statement as it is written: its header with prefix, then the field line, prefix and value included.
 *
 * Spelled the way the file spells it, because the point of a found row is to be recognised in the file it opens.
 *
 * @param declaration - The statement to spell.
 * @returns The statement's text.
 */
export function describeDeclaration(declaration: LtxDeclaration): string {
  const header: string = declaration.section === null ? "" : `${declaration.sectionOperation}[${declaration.section}]`;

  if (declaration.kind === "section") {
    return declaration.parents.length ? `${header}:${declaration.parents.join(", ")}` : header;
  }

  const field: string =
    declaration.value === null
      ? `${declaration.operation}${declaration.name}`
      : `${declaration.operation}${declaration.name} = ${declaration.value}`;

  // A key written before the first header belongs to no section, and is shown as the bare line it is.
  return header ? `${header} ${field}` : field;
}
