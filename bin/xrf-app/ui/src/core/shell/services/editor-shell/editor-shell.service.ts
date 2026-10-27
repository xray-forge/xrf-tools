import { Injectable, OnDeprovision } from "@wirestate/core";
import { BoundAction, RefObservable, runInAction } from "@wirestate/mobx";
import { EMPTY_ARRAY } from "@xrf/collections";
import { Nullable } from "@xrf/types";

import { IEditorPanel } from "@/core/shell/editor-shell/editor-panel";
import { isSameStatusSegment, TEditorStatusSegment } from "@/core/shell/editor-shell/editor-status-segment";
import { IMenuContribution } from "@/core/shell/menu/lib";

interface IEditorShellRegistration<T> {
  owner: string;
  tool: string;
  values: ReadonlyArray<T>;
}

/** Holds the active editor's shell contributions independently of its React tree. */
@Injectable()
export class EditorShellService {
  @RefObservable()
  private status: Nullable<IEditorShellRegistration<TEditorStatusSegment>> = null;

  // Panel descriptors contain React elements and render closures; observe replacement without transforming them.
  @RefObservable()
  private panels: Nullable<IEditorShellRegistration<IEditorPanel>> = null;

  // Unlike status and panels, menus take many owners at once - the shell, a tool, a part inside it - kept in the order
  // each first published, which is the order their entries draw in within a section.
  @RefObservable()
  private menus: ReadonlyArray<IEditorShellRegistration<IMenuContribution>> = EMPTY_ARRAY;

  @OnDeprovision()
  public onDeprovision(): void {
    runInAction(() => {
      this.status = null;
      this.panels = null;
      this.menus = EMPTY_ARRAY;
    });
  }

  /** Returns status only for the tool currently being rendered. */
  public getStatus(tool: string): ReadonlyArray<TEditorStatusSegment> {
    return this.status?.tool === tool ? this.status.values : EMPTY_ARRAY;
  }

  /** Returns panels only for the tool whose container can render them. */
  public getPanels(tool: string): ReadonlyArray<IEditorPanel> {
    return this.panels?.tool === tool ? this.panels.values : EMPTY_ARRAY;
  }

  /** Returns menu entries only for the tool whose container can run them, in publication order. */
  public getMenus(tool: string): ReadonlyArray<IMenuContribution> {
    return this.menus
      .filter((it: IEditorShellRegistration<IMenuContribution>) => it.tool === tool)
      .flatMap((it: IEditorShellRegistration<IMenuContribution>) => it.values);
  }

  /** Publishes status, retaining its snapshot when the same owner repeats unchanged segments. */
  @BoundAction()
  public publishStatus(owner: string, tool: string, segments: ReadonlyArray<TEditorStatusSegment>): void {
    const previous = this.status;

    if (
      previous?.owner === owner &&
      previous.tool === tool &&
      previous.values.length === segments.length &&
      previous.values.every((value: TEditorStatusSegment, index: number) => isSameStatusSegment(value, segments[index]))
    ) {
      return;
    }

    this.status = { owner, tool, values: [...segments] };
  }

  /** Releases only the status published by this registration. */
  @BoundAction()
  public releaseStatus(owner: string): void {
    if (this.status?.owner === owner) {
      this.status = null;
    }
  }

  /** Publishes a panel snapshot, preserving descriptor and renderer identities. */
  @BoundAction()
  public publishPanels(owner: string, tool: string, panels: ReadonlyArray<IEditorPanel>): void {
    this.panels = { owner, tool, values: [...panels] };
  }

  /** Releases only the panels published by this registration. */
  @BoundAction()
  public releasePanels(owner: string): void {
    if (this.panels?.owner === owner) {
      this.panels = null;
    }
  }

  /** Publishes one owner's menu entries, keeping the place it first published at. */
  @BoundAction()
  public publishMenus(owner: string, tool: string, contributions: ReadonlyArray<IMenuContribution>): void {
    const registration: IEditorShellRegistration<IMenuContribution> = {
      owner,
      tool,
      values: [...contributions],
    };
    const index: number = this.menus.findIndex((it: IEditorShellRegistration<IMenuContribution>) => it.owner === owner);

    this.menus =
      index < 0
        ? [...this.menus, registration]
        : this.menus.map((it: IEditorShellRegistration<IMenuContribution>, at: number) =>
            at === index ? registration : it
          );
  }

  /** Releases only the menu entries published by this registration. */
  @BoundAction()
  public releaseMenus(owner: string): void {
    this.menus = this.menus.filter((it: IEditorShellRegistration<IMenuContribution>) => it.owner !== owner);
  }
}
