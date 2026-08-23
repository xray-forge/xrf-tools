import { ExportDescriptor } from "@/core/bindings/types/xrf-export";
import { TCallableExportDescriptor } from "@/core/exports";
import { IPathTreeItem, toDirectoryItemId, toFileItemId } from "@/core/ui/tree/path-tree";

export const ROOT_EXPORT_GROUP_ID: string = "group:root";

export interface IExportGroup {
  id: string;
  label: string;
  declarations: Array<ExportDescriptor>;
}

/**
 * Groups export declarations by the namespace before their first dot.
 *
 * @param declarations - Export declarations to group.
 * @returns Namespace groups sorted by label, with root declarations first.
 */
export function groupExports(declarations: ReadonlyArray<ExportDescriptor>): Array<IExportGroup> {
  const groups: Map<string, IExportGroup> = new Map();

  for (const declaration of declarations) {
    const separator: number = declaration.name.indexOf(".");
    const isRoot: boolean = separator < 0;
    const namespace: string = isRoot ? "" : declaration.name.slice(0, separator);
    const id: string = isRoot ? ROOT_EXPORT_GROUP_ID : `group:namespace:${namespace}`;
    const group: IExportGroup = groups.get(id) ?? {
      id,
      label: isRoot ? "~" : namespace,
      declarations: [],
    };

    group.declarations.push(declaration);
    groups.set(id, group);
  }

  return Array.from(groups.values())
    .sort((left: IExportGroup, right: IExportGroup) => {
      if (left.id === ROOT_EXPORT_GROUP_ID) {
        return -1;
      }

      if (right.id === ROOT_EXPORT_GROUP_ID) {
        return 1;
      }

      return left.label.localeCompare(right.label);
    })
    .map((group: IExportGroup) => ({
      ...group,
      declarations: [...group.declarations].sort((left: ExportDescriptor, right: ExportDescriptor) =>
        left.name.localeCompare(right.name)
      ),
    }));
}

/**
 * Turn namespace groups into the shared explorer tree shape.
 *
 * @param groups - Namespace groups to render.
 * @returns Tree items in the order the groups were sorted into.
 */
export function exportGroupsToTree(groups: ReadonlyArray<IExportGroup>): Array<IPathTreeItem<ExportDescriptor>> {
  return groups.map((group: IExportGroup) => {
    const namespace: string = group.id === ROOT_EXPORT_GROUP_ID ? "" : group.label;

    return {
      id: toDirectoryItemId(namespace),
      label: `${group.label} (${group.declarations.length})`,
      path: namespace,
      kind: "directory",
      children: group.declarations.map((declaration: ExportDescriptor) => ({
        id: toFileItemId(declaration.name),
        label: declaration.name,
        path: declaration.name,
        kind: "file",
        payload: declaration,
      })),
    };
  });
}

export function getExportSearchText(declaration: ExportDescriptor): string {
  const documentation: Array<string> = [declaration.name, declaration.description ?? ""];

  if (declaration.kind === "callable") {
    const callable: TCallableExportDescriptor = declaration;

    documentation.push(callable.returns.description ?? "");
    documentation.push(...callable.parameters.map((parameter) => parameter.description ?? ""));
  }

  return documentation.join("\n").toLocaleLowerCase();
}
