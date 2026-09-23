import { Maybe } from "@xrf/types";

/** What a key nobody uses answers. */
const NOBODY: ReadonlySet<never> = new Set();

/**
 * Who uses each key, so a put touches its users and nothing else.
 */
export class KeyedUsers<T> {
  private readonly users: Map<string, Set<T>> = new Map();

  /**
   * @param key - The key used.
   * @returns Everyone using it.
   */
  public get(key: string): ReadonlySet<T> {
    return this.users.get(key) ?? NOBODY;
  }

  public add(key: string, user: T): void {
    let users: Maybe<Set<T>> = this.users.get(key);

    if (!users) {
      users = new Set();
      this.users.set(key, users);
    }

    users.add(user);
  }

  public delete(key: string, user: T): void {
    const users: Maybe<Set<T>> = this.users.get(key);

    users?.delete(user);

    if (users && !users.size) {
      this.users.delete(key);
    }
  }

  public clear(): void {
    this.users.clear();
  }
}
