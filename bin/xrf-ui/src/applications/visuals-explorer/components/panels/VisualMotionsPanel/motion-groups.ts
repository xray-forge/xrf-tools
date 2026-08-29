import { ITreeNode } from "@/core/ui/tree/tree-node";
import { Optional } from "@/lib/types/general";

/** Separates a motion name's tokens, which is the only structure the format gives them. */
const NAME_SEPARATOR: string = "_";

/** Prefix of a group node's id, so a family and a motion of the same spelling cannot collide. */
const GROUP_ID_PREFIX: string = "family:";

/** Prefix of a motion node's id. */
const MOTION_ID_PREFIX: string = "motion:";

/**
 * @param name - Motion name to address.
 * @returns The tree id a motion is drawn under.
 */
export function toMotionNodeId(name: string): string {
  return `${MOTION_ID_PREFIX}${name}`;
}

/**
 * @param id - Tree id a row carries.
 * @returns The motion the row names, or null for a family row.
 */
export function getMotionNodeName(id: string): string | null {
  return id.startsWith(MOTION_ID_PREFIX) ? id.slice(MOTION_ID_PREFIX.length) : null;
}

/**
 * Groups motion names by the token they start with.
 *
 * @param names - Motion names, in the order the backend listed them.
 * @returns Family nodes and ungrouped motions, ordered by label; each family keeps its listed order.
 */
export function groupMotionNames(names: ReadonlyArray<string>): Array<ITreeNode<string>> {
  const families: Map<string, Array<string>> = new Map();

  for (const name of names) {
    const token: string = name.split(NAME_SEPARATOR)[0] || name;
    const members: Optional<Array<string>> = families.get(token);

    if (members) {
      members.push(name);
    } else {
      families.set(token, [name]);
    }
  }

  const nodes: Array<ITreeNode<string>> = [...families].map(([token, members]: [string, Array<string>]) =>
    members.length > 1
      ? {
          children: members.map(toMotionNode),
          id: `${GROUP_ID_PREFIX}${token}`,
          label: `${token} (${members.length})`,
        }
      : toMotionNode(members[0])
  );

  return nodes.sort((left: ITreeNode<string>, right: ITreeNode<string>) => left.label.localeCompare(right.label));
}

/**
 * Names every family a set of motions produces, for expanding what a filter matched.
 *
 * @param nodes - Nodes to read.
 * @returns Ids of the nodes that hold members.
 */
export function listMotionGroupIds(nodes: ReadonlyArray<ITreeNode<string>>): Array<string> {
  return nodes.filter((node: ITreeNode<string>) => node.children?.length).map((node: ITreeNode<string>) => node.id);
}

/**
 * @param name - Motion the node stands for.
 * @returns A leaf carrying the name as both its label and its payload.
 */
function toMotionNode(name: string): ITreeNode<string> {
  return { id: toMotionNodeId(name), label: name, payload: name };
}
