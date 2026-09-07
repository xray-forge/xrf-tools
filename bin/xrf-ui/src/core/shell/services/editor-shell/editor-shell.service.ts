import { Injectable, OnDeprovision } from "@wirestate/core";
import { BoundAction, RefObservable, runInAction } from "@wirestate/mobx";

import { IEditorPanel } from "@/core/shell/editor-shell/editor-panel";
import { Nullable } from "@/lib/types/general";

interface IEditorShellRegistration<T> {
  owner: string;
  application: string;
  values: ReadonlyArray<T>;
}

const NO_STATUS: ReadonlyArray<string> = [];
const NO_PANELS: ReadonlyArray<IEditorPanel> = [];

/** Holds the active editor's shell contributions independently of its React tree. */
@Injectable()
export class EditorShellService {
  @RefObservable()
  private status: Nullable<IEditorShellRegistration<string>> = null;

  // Panel descriptors contain React elements and render closures; observe replacement without transforming them.
  @RefObservable()
  private panels: Nullable<IEditorShellRegistration<IEditorPanel>> = null;

  @OnDeprovision()
  public onDeprovision(): void {
    runInAction(() => {
      this.status = null;
      this.panels = null;
    });
  }

  /** Returns status only for the application currently being rendered. */
  public getStatus(application: string): ReadonlyArray<string> {
    return this.status?.application === application ? this.status.values : NO_STATUS;
  }

  /** Returns panels only for the application whose container can render them. */
  public getPanels(application: string): ReadonlyArray<IEditorPanel> {
    return this.panels?.application === application ? this.panels.values : NO_PANELS;
  }

  /** Publishes status, retaining its snapshot when the same owner repeats unchanged text. */
  @BoundAction()
  public publishStatus(owner: string, application: string, segments: ReadonlyArray<string>): void {
    const previous = this.status;

    if (
      previous?.owner === owner &&
      previous.application === application &&
      previous.values.length === segments.length &&
      previous.values.every((value, index) => value === segments[index])
    ) {
      return;
    }

    this.status = { owner, application, values: [...segments] };
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
  public publishPanels(owner: string, application: string, panels: ReadonlyArray<IEditorPanel>): void {
    this.panels = { owner, application, values: [...panels] };
  }

  /** Releases only the panels published by this registration. */
  @BoundAction()
  public releasePanels(owner: string): void {
    if (this.panels?.owner === owner) {
      this.panels = null;
    }
  }
}
