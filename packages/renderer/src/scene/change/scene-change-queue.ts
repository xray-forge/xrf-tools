import { Maybe, Nullable } from "@xrf/types";
import { BufferGeometry, Mesh } from "three/webgpu";

import { ISceneChange } from "#/scene/change/scene-change";
import { ISceneChangeHandler } from "#/scene/change/scene-change-handler";

/**
 * Objects one settle applies before leaving the rest to the next frame: each draws for the first time in the frame
 * after, where three builds its render state, and a whole level's worth of those was one long frame.
 */
const APPLIED_PER_SETTLE: number = 32;

/**
 * The changes a consumer made that do not draw yet, oldest first.
 *
 * A transaction is one change. A change that touches an object an older change still waits on takes the older one
 * with it, so neither applies without the other. What a change adds is only valid inside a transaction.
 */
export class SceneChangeQueue<T> {
  private readonly handler: ISceneChangeHandler<T>;
  private readonly budget: number;
  private readonly changes: Array<ISceneChange<T>> = [];
  /** The change each waiting object waits in. */
  private readonly waiting: Map<T, ISceneChange<T>> = new Map();
  /** How deep in `transact` the queue is: nothing settles until the outermost one ends. */
  private depth: number = 0;
  /** The change the running transaction adds to, made on its first use. */
  private open: Nullable<ISceneChange<T>> = null;

  /**
   * @param handler - What changes are applied through.
   * @param budget - Objects one settle applies, at the least one whole change.
   */
  public constructor(handler: ISceneChangeHandler<T>, budget: number = APPLIED_PER_SETTLE) {
    this.handler = handler;
    this.budget = budget;
  }

  /** Whether any object waits: for its materials, its textures, or its turn. */
  public get hasPending(): boolean {
    return this.waiting.size > 0;
  }

  /** Every waiting object, oldest change first. */
  public get pending(): Iterable<T> {
    return this.changes.flatMap((change: ISceneChange<T>) => [...change.objects]);
  }

  /** Meshes still drawn for objects already released. */
  public get leaving(): Iterable<Mesh> {
    return this.changes.flatMap((change: ISceneChange<T>) => change.leaving);
  }

  /**
   * Makes a run of changes one change: it applies only once all of it can draw.
   *
   * @param change - What to change.
   */
  public transact(change: () => void): void {
    this.depth += 1;

    try {
      change();
    } finally {
      this.depth -= 1;
    }

    if (!this.depth) {
      this.open = null;
      this.settle();
    }
  }

  /** Applies what became ready since the last settle, outside any transaction. */
  public advance(): void {
    if (this.changes.length && !this.depth) {
      this.settle();
    }
  }

  /**
   * @param object - An object to draw as it is now put, once the running change applies.
   */
  public enlist(object: T): void {
    const into: ISceneChange<T> = this.join(object);

    this.waiting.set(object, into);
    into.objects.add(object);
  }

  /**
   * @param object - An object released, which never draws again.
   * @param meshes - What drew it, drawn until the running change applies.
   * @param geometries - What only it drew, disposed then.
   */
  public withdraw(object: T, meshes: Iterable<Mesh>, geometries: Iterable<BufferGeometry>): void {
    // Whatever it waited in goes with its release, or a geometry that change let go of would be disposed while the
    // meshes leaving here still draw it.
    const into: ISceneChange<T> = this.join(object);

    into.objects.delete(object);
    this.waiting.delete(object);
    into.leaving.push(...meshes);

    for (const geometry of geometries) {
      into.geometries.add(geometry);
    }
  }

  /**
   * @param geometry - A geometry no longer put, disposed once the running change applies.
   */
  public retireGeometry(geometry: BufferGeometry): void {
    this.current.geometries.add(geometry);
  }

  /**
   * @param key - A texture released: still sampled by whatever draws until the changes before this one apply, so let
   *   go of only then.
   */
  public releaseTexture(key: string): void {
    this.current.textures.add(key);
  }

  /**
   * @param key - A texture put again, which supersedes any release of it still waiting.
   */
  public keepTexture(key: string): void {
    this.changes.forEach((change: ISceneChange<T>) => change.textures.delete(key));
  }

  /** Lets go of everything waiting, applying none of it. */
  public dispose(): void {
    for (const change of this.changes) {
      change.leaving.forEach((mesh: Mesh) => mesh.removeFromParent());
      change.geometries.forEach((geometry: BufferGeometry) => geometry.dispose());
    }

    this.changes.length = 0;
    this.waiting.clear();
    this.open = null;
  }

  /** The change the running transaction adds to. */
  private get current(): ISceneChange<T> {
    if (!this.open) {
      this.open = { geometries: new Set(), leaving: [], objects: new Set(), textures: new Set() };
      this.changes.push(this.open);
    }

    return this.open;
  }

  /** The running change, with whatever change the object already waited in merged into it. */
  private join(object: T): ISceneChange<T> {
    const into: ISceneChange<T> = this.current;
    const from: Maybe<ISceneChange<T>> = this.waiting.get(object);

    if (from && from !== into) {
      this.merge(from, into);
    }

    return into;
  }

  /** Makes an earlier change part of a later one, so neither applies without the other. */
  private merge(from: ISceneChange<T>, into: ISceneChange<T>): void {
    for (const object of from.objects) {
      this.waiting.set(object, into);
      into.objects.add(object);
    }

    into.leaving.push(...from.leaving);
    from.geometries.forEach((geometry: BufferGeometry) => into.geometries.add(geometry));
    from.textures.forEach((key: string) => into.textures.add(key));
    this.changes.splice(this.changes.indexOf(from), 1);
  }

  /**
   * Applies every change that can draw now, each whole: what a consumer changed together appears together, and what
   * it released goes in the same frame. Changes wait only for themselves - a sector whose materials are compiled does
   * not wait behind one whose are not - except one letting textures go, which waits for every change before it, since
   * any of those may be about to sample them. Past the budget the rest wait for the next settle.
   */
  private settle(): void {
    let applied: number = 0;
    let isBlocked: boolean = false;

    for (const change of [...this.changes]) {
      if (applied && applied + change.objects.size > this.budget) {
        break;
      }

      if (!this.canApply(change) || (isBlocked && change.textures.size)) {
        isBlocked = true;
        continue;
      }

      this.changes.splice(this.changes.indexOf(change), 1);
      applied += change.objects.size;

      change.objects.forEach((object: T) => {
        this.waiting.delete(object);
        this.handler.apply(object);
      });
      change.leaving.forEach((mesh: Mesh) => mesh.removeFromParent());
      change.geometries.forEach((geometry: BufferGeometry) => geometry.dispose());
      change.textures.forEach((key: string) => this.handler.releaseTexture(key));
    }

    this.handler.settled();
  }

  private canApply(change: ISceneChange<T>): boolean {
    for (const object of change.objects) {
      if (!this.handler.canApply(object)) {
        return false;
      }
    }

    return true;
  }
}
