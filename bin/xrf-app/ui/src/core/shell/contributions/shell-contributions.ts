import { BoundAction, makeObservable, RefObservable } from "@wirestate/mobx";
import { EMPTY_ARRAY } from "@xrf/collections";
import { SHELL_SESSION_TOOL } from "@xrf/ipc/session";
import { Maybe } from "@xrf/types";

import { EToolId } from "@/core/routing/tool";

/** The key a contribution is drawn under: the tool it belongs to, or the shell's own, which the launcher shares. */
export type TShellContributionTool = EToolId | typeof SHELL_SESSION_TOOL;

/** What one owner contributes for one tool. */
interface IShellContribution<T> {
  readonly owner: string;
  readonly tool: TShellContributionTool;
  readonly values: ReadonlyArray<T>;
}

/**
 * What any number of owners add to one part of the shell, such as its panels, for each tool. Owners are kept in the
 * order each first published, which is the order their values draw in.
 */
export class ShellContributions<T> {
  // Values may hold React elements and render closures; observe replacement without transforming them.
  @RefObservable()
  private contributions: ReadonlyArray<IShellContribution<T>> = EMPTY_ARRAY;

  /**
   * @param isSame - Whether two values draw alike, so an owner repeating them changes nothing; identity by default.
   */
  public constructor(private readonly isSame: (first: T, second: T) => boolean = Object.is) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  /**
   * @param tool - The tool whose values are read.
   * @returns Every owner's values for that tool, in the order the owners first published.
   */
  public get(tool: TShellContributionTool): ReadonlyArray<T> {
    return this.contributions
      .filter((it: IShellContribution<T>) => it.tool === tool)
      .flatMap((it: IShellContribution<T>) => it.values);
  }

  /**
   * Publishes one owner's values, in the place it first published at. Repeating values that draw alike for the same
   * tool changes nothing, so whoever draws them is not redrawn.
   *
   * @param owner - Identity of the publishing registration.
   * @param tool - The tool the values belong to.
   * @param values - The values, copied so the caller's array may change afterwards.
   */
  @BoundAction()
  public publish(owner: string, tool: TShellContributionTool, values: ReadonlyArray<T>): void {
    const index: number = this.contributions.findIndex((it: IShellContribution<T>) => it.owner === owner);
    const previous: Maybe<IShellContribution<T>> = this.contributions[index];

    if (
      previous?.tool === tool &&
      previous.values.length === values.length &&
      previous.values.every((value: T, at: number) => this.isSame(value, values[at]))
    ) {
      return;
    }

    const contribution: IShellContribution<T> = { owner, tool, values: [...values] };

    this.contributions =
      index < 0
        ? [...this.contributions, contribution]
        : this.contributions.map((it: IShellContribution<T>, at: number) => (at === index ? contribution : it));
  }

  /**
   * @param owner - Identity of the registration whose values go; any other owner's stay.
   */
  @BoundAction()
  public release(owner: string): void {
    if (this.contributions.some((it: IShellContribution<T>) => it.owner === owner)) {
      this.contributions = this.contributions.filter((it: IShellContribution<T>) => it.owner !== owner);
    }
  }

  /** Forgets every owner's values. */
  @BoundAction()
  public clear(): void {
    this.contributions = EMPTY_ARRAY;
  }
}
