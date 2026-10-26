/**
 * One node of a tree, whatever the tree is built from.
 */
export interface ITreeNode<T> {
  id: string;
  label: string;
  /** What the consumer identified this node by, where it stands for something. */
  payload?: T;
  children?: ReadonlyArray<ITreeNode<T>>;
}
