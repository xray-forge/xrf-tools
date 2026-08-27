import { describe, expect, it } from "@jest/globals";

import {
  buildDialogGraph,
  DIALOG_NODE_ID,
  EDialogGraphNodeType,
  IDialogGraph,
  IDialogGraphNodeData,
} from "@/applications/dialogs-editor/lib/dialog-graph";
import { DialogDescriptor, DialogPhraseDescriptor } from "@/core/bindings/types/xrf-dialog";
import { TGraphEdge, TGraphNode } from "@/core/graph/lib";

function phrase(overrides: Partial<DialogPhraseDescriptor> = {}): DialogPhraseDescriptor {
  return {
    id: "0",
    textKey: null,
    text: null,
    isFinal: false,
    isInPhraseList: true,
    next: [],
    elements: [],
    ...overrides,
  };
}

function dialog(phrases: Array<DialogPhraseDescriptor>, overrides: Partial<DialogDescriptor> = {}): DialogDescriptor {
  return {
    logicalPath: "configs\\gameplay\\dialogs.xml",
    id: "trader",
    priority: null,
    elements: [],
    language: "eng",
    phrases,
    ...overrides,
  };
}

function nodeOf(graph: IDialogGraph, id: string): TGraphNode<IDialogGraphNodeData> {
  const found: TGraphNode<IDialogGraphNodeData> | undefined = graph.nodes.find(
    (node: TGraphNode<IDialogGraphNodeData>) => node.id === id
  );

  if (!found) {
    throw new Error(`Expected a node ${id}`);
  }

  return found;
}

describe("buildDialogGraph", () => {
  it("draws nothing for no dialog", () => {
    expect(buildDialogGraph(null)).toEqual({ edges: [], nodes: [] });
  });

  it("roots the conversation at the dialog and its entry phrase", () => {
    const graph: IDialogGraph = buildDialogGraph(dialog([phrase({ id: "0", next: ["1"] }), phrase({ id: "1" })]));

    expect(nodeOf(graph, DIALOG_NODE_ID).type).toBe(EDialogGraphNodeType.DIALOG);
    expect(nodeOf(graph, "0").type).toBe(EDialogGraphNodeType.PHRASE);
    expect(graph.edges.map((edge: TGraphEdge) => `${edge.source}->${edge.target}`)).toEqual([
      `${DIALOG_NODE_ID}->0`,
      "0->1",
    ]);
  });

  it("draws no entry edge for a dialog that declares no phrase zero", () => {
    // Built from script at runtime. An edge to a phrase nobody declared would place an endpoint that
    // does not exist.
    const graph: IDialogGraph = buildDialogGraph(dialog([phrase({ id: "greet" })]));

    expect(graph.edges).toEqual([]);
    expect(graph.nodes).toHaveLength(2);
  });

  it("numbers outgoing edges only where there is a choice to order", () => {
    // `next` order is what the player is offered, so a branch is numbered. A single option has no
    // sequence worth reading, and labelling it would be noise on every linear conversation.
    const graph: IDialogGraph = buildDialogGraph(
      dialog([phrase({ id: "0", next: ["a", "b"] }), phrase({ id: "a", next: ["b"] }), phrase({ id: "b" })])
    );

    const branch: Array<TGraphEdge> = graph.edges.filter((edge: TGraphEdge) => edge.source === "0");

    expect(branch.map((edge: TGraphEdge) => edge.label)).toEqual(["1", "2"]);
    expect(graph.edges.find((edge: TGraphEdge) => edge.source === "a")?.label).toBeUndefined();
  });

  it("keeps both edges when a phrase lists the same target twice", () => {
    // Duplicated options occupy two slots in what the player sees, so collapsing them would renumber
    // every option after them.
    const graph: IDialogGraph = buildDialogGraph(dialog([phrase({ id: "0", next: ["1", "1"] }), phrase({ id: "1" })]));

    const ids: Array<string> = graph.edges
      .filter((edge: TGraphEdge) => edge.source === "0")
      .map((edge: TGraphEdge) => String(edge.id));

    expect(ids).toHaveLength(2);
    expect(new Set(ids).size).toBe(2);
  });

  it("drops an edge to a phrase the dialog does not declare", () => {
    // A broken link. The canvas cannot place an endpoint that does not exist; validation reports it.
    const graph: IDialogGraph = buildDialogGraph(dialog([phrase({ id: "0", next: ["missing"] })]));

    // The entry edge stands, because phrase zero is declared. Only the dangling one is dropped.
    expect(graph.edges.map((edge: TGraphEdge) => `${edge.source}->${edge.target}`)).toEqual([`${DIALOG_NODE_ID}->0`]);
    expect(graph.nodes.map((node: TGraphNode) => node.id)).toEqual([DIALOG_NODE_ID, "0"]);
  });

  it("does not invent an edge to an unreferenced phrase", () => {
    // Drawing it from the dialog root would make an unreachable phrase look reachable, which is the
    // one thing validation later has to be able to say about it.
    const graph: IDialogGraph = buildDialogGraph(dialog([phrase({ id: "0" }), phrase({ id: "orphan" })]));

    expect(graph.edges.map((edge: TGraphEdge) => edge.target)).toEqual(["0"]);
    expect(nodeOf(graph, "orphan").data.hasIncoming).toBe(false);
    expect(nodeOf(graph, "0").data.hasIncoming).toBe(false);
  });

  it("marks a phrase as having something leading to it", () => {
    const graph: IDialogGraph = buildDialogGraph(dialog([phrase({ id: "0", next: ["1"] }), phrase({ id: "1" })]));

    expect(nodeOf(graph, "1").data.hasIncoming).toBe(true);
  });

  it("labels a node with its line, its key, or a stated absence", () => {
    const graph: IDialogGraph = buildDialogGraph(
      dialog([
        phrase({ id: "0", textKey: "trader_hello", text: "Hello, stalker" }),
        phrase({ id: "1", textKey: "trader_bye", text: null }),
        phrase({ id: "2", textKey: null, text: null }),
      ])
    );

    expect(nodeOf(graph, "0").data).toMatchObject({ isUnresolved: false, label: "Hello, stalker" });
    // The key stands in, so a writer sees which one is missing text rather than an empty box.
    expect(nodeOf(graph, "1").data).toMatchObject({ isUnresolved: true, label: "trader_bye" });
    expect(nodeOf(graph, "2").data).toMatchObject({ isUnresolved: true, label: "(built from script)" });
  });

  it("badges behaviour and not the content the graph already draws", () => {
    const graph: IDialogGraph = buildDialogGraph(
      dialog([
        phrase({
          id: "0",
          next: ["0"],
          elements: [
            { name: "text", kind: "text", value: "trader_hello" },
            { name: "next", kind: "next", value: "0" },
            { name: "give_info", kind: "giveInfo", value: "met" },
            { name: "has_info", kind: "hasInfo", value: "a" },
            { name: "has_info", kind: "hasInfo", value: "b" },
          ],
        }),
      ])
    );

    // `text` is the label and `next` is an edge, so badging them would repeat what is already drawn.
    // Repeats are kept: two info gates are two conditions, and a set would silently drop one.
    expect(nodeOf(graph, "0").data.badges).toEqual(["give_info", "has_info", "has_info"]);
  });

  it("places every node and lays a linear conversation out in reading order", () => {
    const graph: IDialogGraph = buildDialogGraph(
      dialog([phrase({ id: "0", next: ["1"] }), phrase({ id: "1", next: ["2"] }), phrase({ id: "2" })])
    );

    // Top-down, so a later phrase sits below the one leading to it. Nothing is left at the origin,
    // which is what an unplaced node looks like.
    function y(id: string): number {
      return nodeOf(graph, id).position.y;
    }

    expect(y(DIALOG_NODE_ID)).toBeLessThan(y("0"));
    expect(y("0")).toBeLessThan(y("1"));
    expect(y("1")).toBeLessThan(y("2"));
  });

  it("lays the same dialog out identically every time", () => {
    // Recomputed on open rather than persisted, so determinism is what stands in for saved positions.
    const source: DialogDescriptor = dialog([
      phrase({ id: "0", next: ["a", "b"] }),
      phrase({ id: "a", next: ["b"] }),
      phrase({ id: "b" }),
      phrase({ id: "orphan" }),
    ]);

    expect(buildDialogGraph(source).nodes.map((node: TGraphNode) => node.position)).toEqual(
      buildDialogGraph(source).nodes.map((node: TGraphNode) => node.position)
    );
  });

  it("terminates on a conversation that loops back on itself", () => {
    // Dagre breaks cycles internally, so a dialog offering "ask again" ranks like any other.
    const graph: IDialogGraph = buildDialogGraph(
      dialog([phrase({ id: "0", next: ["1"] }), phrase({ id: "1", next: ["0"] })])
    );

    expect(graph.nodes).toHaveLength(3);
    expect(graph.edges).toHaveLength(3);
  });
});

describe("buildDialogGraph at the worst case in the data", () => {
  /** `about_quests_dialog_stalkers`: 96 phrases, a fan of options off the entry, each with a reply. */
  function largest(): DialogDescriptor {
    const options: Array<string> = Array.from({ length: 47 }, (_, index: number) => `q${index}`);

    return dialog([
      phrase({ id: "0", next: options, textKey: "about_quests_0", text: "What have you got?" }),
      ...options.flatMap((option: string) => [
        phrase({ id: option, next: [`${option}r`], textKey: option, text: `Question ${option}` }),
        phrase({ id: `${option}r`, isFinal: true, textKey: `${option}r`, text: `Answer ${option}` }),
      ]),
    ]);
  }

  it("lays out ninety-six phrases into ranks without leaving anything at the origin", () => {
    const graph: IDialogGraph = buildDialogGraph(largest());

    expect(graph.nodes).toHaveLength(96);
    // 47 options off the entry, 47 replies, plus the dialog's own edge to phrase zero.
    expect(graph.edges).toHaveLength(95);

    const ranks: Set<number> = new Set(graph.nodes.map((node: TGraphNode) => node.position.y));

    // Dialog, entry, options, replies. A single rank would mean nothing was ranked at all.
    expect(ranks.size).toBe(4);
    expect(graph.nodes.every((node: TGraphNode) => Number.isFinite(node.position.x))).toBe(true);
  });

  it("numbers all forty-seven options, because that order is what the player scrolls through", () => {
    const graph: IDialogGraph = buildDialogGraph(largest());
    const labels: Array<unknown> = graph.edges
      .filter((edge: TGraphEdge) => edge.source === "0")
      .map((edge: TGraphEdge) => edge.label);

    expect(labels).toHaveLength(47);
    expect(labels[0]).toBe("1");
    expect(labels[46]).toBe("47");
  });
});
