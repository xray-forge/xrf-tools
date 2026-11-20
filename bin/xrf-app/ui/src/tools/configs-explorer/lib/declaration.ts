import { ELtxDeclarationStatement, LtxDeclaration } from "@xrf/ipc/types/xrf-ltx-inspect";

/**
 * One found statement as the file spells it: its header with prefix, then the field line with prefix and value.
 *
 * @param declaration - The statement to spell.
 * @returns The statement's text.
 */
export function describeDeclaration(declaration: LtxDeclaration): string {
  const header: string = declaration.section === null ? "" : `${declaration.sectionOperation}[${declaration.section}]`;

  if (declaration.kind === ELtxDeclarationStatement.SECTION) {
    return declaration.parents.length ? `${header}:${declaration.parents.join(", ")}` : header;
  }

  const field: string =
    declaration.value === null
      ? `${declaration.operation}${declaration.name}`
      : `${declaration.operation}${declaration.name} = ${declaration.value}`;

  // A key written before the first header belongs to no section, and is shown as the bare line it is.
  return header ? `${header} ${field}` : field;
}
