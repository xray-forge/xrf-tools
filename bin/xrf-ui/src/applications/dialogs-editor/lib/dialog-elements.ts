import { DialogElementDescriptor } from "@/core/bindings/types/xrf-dialog";

/** One group of elements, titled by what the elements in it do. */
export interface IDialogElementGroup {
  id: string;
  title: string;
  /** What the group means, for a reader who does not already know the engine vocabulary. */
  caption: string;
  elements: Array<DialogElementDescriptor>;
}

/**
 * The groups, in the order a reader asks the questions.
 */
const GROUPS: ReadonlyArray<{
  id: string;
  title: string;
  caption: string;
  kinds: ReadonlyArray<DialogElementDescriptor["kind"]>;
}> = [
  {
    caption: "What has to hold for this to be offered",
    id: "conditions",
    kinds: ["precondition", "hasInfo", "dontHasInfo"],
    title: "Conditions",
  },
  {
    caption: "What selecting it does to the world",
    id: "effects",
    kinds: ["giveInfo", "disableInfo", "action"],
    title: "Effects",
  },
  {
    caption: "Script the engine calls here",
    id: "script",
    kinds: ["initFunc", "scriptText"],
    title: "Script",
  },
  {
    caption: "Not part of the schema, kept as written",
    id: "unrecognised",
    kinds: ["unknown"],
    title: "Unrecognised",
  },
];

/**
 * Group a node's elements by what they do.
 *
 * @param elements - The node's elements, in document order.
 * @returns Non-empty groups, in reading order, each preserving document order within it.
 */
export function groupDialogElements(elements: ReadonlyArray<DialogElementDescriptor>): Array<IDialogElementGroup> {
  return GROUPS.map(({ id, title, caption, kinds }) => ({
    caption,
    elements: elements.filter((element: DialogElementDescriptor) => kinds.includes(element.kind)),
    id,
    title,
  })).filter((group: IDialogElementGroup) => group.elements.length > 0);
}
