import { VisualBone } from "@/core/bindings/types/xrf-visual";
import { ITreeNode } from "@/core/ui/tree/tree-node";
import { Maybe } from "@/lib/types/general";

/** A skeleton node, mutable while the hierarchy is being assembled from parent names. */
interface IBoneTreeNode extends ITreeNode<VisualBone> {
  payload: VisualBone;
  children: Array<IBoneTreeNode>;
}

/**
 * Build the skeleton tree from parent names.
 *
 * OGF stores the hierarchy as names rather than indices, so a bone whose parent is not in the list, or whose parent is
 * itself, would otherwise disappear from the tree. Such a bone is attached at the root instead: a malformed skeleton
 * should still be inspectable.
 *
 * A bone is identified by its name, which is what the viewport highlights and what the visibility mask hides, so the
 * node id is the name rather than a synthetic path.
 *
 * @param bones - Bones as the file lists them, each naming its parent.
 * @returns Root bones, each carrying its descendants.
 */
export function toBoneTree(bones: Array<VisualBone>): Array<ITreeNode<VisualBone>> {
  const nodes: Map<string, IBoneTreeNode> = new Map(
    bones.map((bone: VisualBone) => [bone.name, { id: bone.name, label: bone.name, payload: bone, children: [] }])
  );
  const roots: Array<IBoneTreeNode> = [];

  for (const bone of bones) {
    const node: IBoneTreeNode = nodes.get(bone.name) as IBoneTreeNode;
    const parent: Maybe<IBoneTreeNode> = bone.parent === bone.name ? null : nodes.get(bone.parent);

    if (parent) {
      parent.children.push(node);
    } else {
      roots.push(node);
    }
  }

  return roots;
}
