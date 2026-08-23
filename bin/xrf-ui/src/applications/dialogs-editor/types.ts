export enum EGraphNodeType {
  DIALOG_NODE = "dialog_node",
  PHRASE_NODE = "phrase_node",
}

/**
 * What a dialog node draws.
 *
 * Sample shape until the editor reads real dialog XML: `tags` stands in for the dialog-level
 * conditions (`precondition`, `has_info`, `dont_has_info`) a dialog actually carries.
 */
export interface IDialogNodeData extends Record<string, unknown> {
  label: string;
  tags: Array<string>;
}

/**
 * What a phrase node draws.
 *
 * `tags` stands in for the phrase's own elements. They repeat in real data, so this is a list rather
 * than a fixed set of fields.
 */
export interface IPhraseNodeData extends Record<string, unknown> {
  label: string;
  tags: Array<string>;
}
