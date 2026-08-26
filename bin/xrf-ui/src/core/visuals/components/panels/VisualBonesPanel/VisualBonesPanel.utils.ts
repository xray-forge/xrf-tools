import { TreeViewDefaultItemModelProperties } from "@mui/x-tree-view";

import { VisualBone } from "@/core/bindings/types/xrf-visual";
import { Maybe } from "@/lib/types/general";

/**
 * Build the skeleton tree from parent names.
 *
 * OGF stores the hierarchy as names rather than indices, so a bone whose parent is not in the list, or whose parent is
 * itself, would otherwise disappear from the tree. Such a bone is attached at the root instead: a malformed skeleton
 * should still be inspectable.
 *
 * @param bones - Bones as the file lists them, each naming its parent.
 * @returns Root bones, each carrying its descendants.
 */
export function toBoneTree(bones: Array<VisualBone>): Array<TreeViewDefaultItemModelProperties> {
  const nodes: Map<string, TreeViewDefaultItemModelProperties> = new Map(
    bones.map((bone) => [bone.name, { id: bone.name, label: bone.name, children: [] }])
  );
  const roots: Array<TreeViewDefaultItemModelProperties> = [];

  for (const bone of bones) {
    const node: TreeViewDefaultItemModelProperties = nodes.get(bone.name) as TreeViewDefaultItemModelProperties;
    const parent: Maybe<TreeViewDefaultItemModelProperties> = bone.parent === bone.name ? null : nodes.get(bone.parent);

    if (parent) {
      parent.children?.push(node);
    } else {
      roots.push(node);
    }
  }

  return roots;
}
