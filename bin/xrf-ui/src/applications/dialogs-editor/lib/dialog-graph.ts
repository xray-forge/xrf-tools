import { DialogDescriptor, DialogElementDescriptor, DialogPhraseDescriptor } from "@/core/bindings/types/xrf-dialog";
import { layoutGraphNodes, TGraphEdge, TGraphNode } from "@/core/graph/lib";
import { Nullable } from "@/lib/types/general";

/** Node kinds the dialog canvas draws. */
export enum EDialogGraphNodeType {
  DIALOG = "dialog",
  PHRASE = "phrase",
}

/** The id of the single dialog node, which no phrase can collide with: phrase ids never contain `:`. */
export const DIALOG_NODE_ID: string = "dialog:root";

/** The phrase the engine starts a conversation at. */
const ENTRY_PHRASE_ID: string = "0";

/**
 * Element kinds worth a badge on a node.
 *
 * Behaviour, not content. `text` and `next` are already the node's line and its edges, so badging them
 * would repeat what the graph draws; `container` is structure the reader classified, not something the
 * file says about the phrase.
 */
const BADGE_KINDS: ReadonlySet<DialogElementDescriptor["kind"]> = new Set<DialogElementDescriptor["kind"]>([
  "action",
  "disableInfo",
  "dontHasInfo",
  "giveInfo",
  "hasInfo",
  "initFunc",
  "isFinal",
  "precondition",
  "scriptText",
]);

export interface IDialogGraphNodeData extends Record<string, unknown> {
  /** The line to read: resolved text where there is any, the key where there is not. */
  label: string;
  /** Whether `label` is the key standing in for a line nothing resolved. */
  isUnresolved: boolean;
  /** Phrase id, or the dialog id on the root node. */
  name: string;
  /** Element names the node carries, in document order and repeated as the file repeats them. */
  badges: Array<string>;
  /** Whether selecting this phrase ends the conversation. */
  isFinal: boolean;
  /** Whether anything leads here. False for the entry phrase and for a phrase nothing references. */
  hasIncoming: boolean;
}

export interface IDialogGraph {
  nodes: Array<TGraphNode<IDialogGraphNodeData>>;
  edges: Array<TGraphEdge>;
}

/** Element names a node badges, in document order, repeats included. */
function toBadges(elements: ReadonlyArray<DialogElementDescriptor>): Array<string> {
  return elements
    .filter((element: DialogElementDescriptor) => BADGE_KINDS.has(element.kind))
    .map((element: DialogElementDescriptor) => element.name);
}

/**
 * What a phrase node reads as.
 *
 * The resolved line where the text tree had one, the key where it did not, and a stated absence where
 * there is no key at all. Never empty: a node with no label is a box a reader cannot identify, and the
 * three cases are exactly what a writer needs to tell apart.
 */
function toLabel(phrase: DialogPhraseDescriptor): { label: string; isUnresolved: boolean } {
  if (phrase.text) {
    return { isUnresolved: false, label: phrase.text };
  }

  if (phrase.textKey) {
    return { isUnresolved: true, label: phrase.textKey };
  }

  return { isUnresolved: true, label: "(built from script)" };
}

/**
 * Turn one dialog into the graph its canvas draws.
 *
 * Pure, and separate from the canvas, because this is where every decision worth testing lives: which
 * edges exist, what a node says, and what order things are laid out in.
 *
 * **No edge is invented.** A phrase nothing references stays unreferenced and dagre parks it in its own
 * component; drawing it from the dialog root instead would make an unreachable phrase look reachable,
 * which is the one thing validation later has to be able to say about it.
 *
 * @param dialog - The dialog to draw, as `get_dialog` described it.
 * @returns Nodes carrying computed positions, and the edges that ranked them.
 */
export function buildDialogGraph(dialog: Nullable<DialogDescriptor>): IDialogGraph {
  if (!dialog) {
    return { edges: [], nodes: [] };
  }

  const phrases: ReadonlyArray<DialogPhraseDescriptor> = dialog.phrases;
  const declared: ReadonlySet<string> = new Set(phrases.map((phrase: DialogPhraseDescriptor) => phrase.id));
  const referenced: Set<string> = new Set<string>();
  const edges: Array<TGraphEdge> = [];

  for (const phrase of phrases) {
    phrase.next.forEach((target: string, index: number) => {
      referenced.add(target);

      // An edge to a phrase the dialog does not declare is a broken link. It is dropped rather than
      // drawn, because the canvas cannot place an endpoint that does not exist — validation reports it.
      if (!declared.has(target)) {
        return;
      }

      edges.push({
        id: `${phrase.id}->${target}#${index}`,
        // Only where there is a choice to order. One option has no sequence worth reading.
        label: phrase.next.length > 1 ? String(index + 1) : undefined,
        source: phrase.id,
        target,
      });
    });
  }

  // The dialog itself, holding the conditions that gate the whole conversation.
  const nodes: Array<TGraphNode<IDialogGraphNodeData>> = [
    {
      data: {
        badges: toBadges(dialog.elements),
        hasIncoming: false,
        isFinal: false,
        isUnresolved: false,
        label: dialog.id,
        name: dialog.id,
      },
      id: DIALOG_NODE_ID,
      position: { x: 0, y: 0 },
      type: EDialogGraphNodeType.DIALOG,
    },
    // Document order, which is what makes the layout of a disconnected phrase stable across runs.
    ...phrases.map((phrase: DialogPhraseDescriptor) => ({
      data: {
        badges: toBadges(phrase.elements),
        hasIncoming: referenced.has(phrase.id),
        isFinal: phrase.isFinal,
        name: phrase.id,
        ...toLabel(phrase),
      },
      id: phrase.id,
      position: { x: 0, y: 0 },
      type: EDialogGraphNodeType.PHRASE,
    })),
  ];

  // The conversation starts at the entry phrase, which the engine fixes as `0`. Attached only when the
  // dialog declares one, so a dialog building its phrases from script draws a lone root rather than an
  // edge to nothing.
  if (declared.has(ENTRY_PHRASE_ID)) {
    edges.unshift({ id: `${DIALOG_NODE_ID}->${ENTRY_PHRASE_ID}`, source: DIALOG_NODE_ID, target: ENTRY_PHRASE_ID });
  }

  return { edges, nodes: layoutGraphNodes(nodes, edges) };
}
