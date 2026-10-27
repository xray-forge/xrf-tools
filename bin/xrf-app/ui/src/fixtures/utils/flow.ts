import { jest } from "@jest/globals";

/** What a stubbed flow answers: the flow's promise, a rejection for one that fails. */
type TFlowStub = (...args: Array<any>) => Promise<unknown>;

/**
 * Stubs a `@LatestFlow` or `@ExclusiveFlow` method of one instance, for a test of what calls it.
 *
 * `jest.spyOn` cannot: the decorator binds the method as a read-only property of the instance it is first read on, and
 * declares it as the generator it is written as while its callers are handed the flow's promise. The stub takes the
 * property's place and answers that promise.
 *
 * @param instance - The instance whose flow is stubbed.
 * @param key - The flow method.
 * @param answer - What each call answers.
 * @returns The stub, to assert on.
 */
export function mockFlow<T extends object>(
  instance: T,
  key: keyof T,
  answer: TFlowStub = async () => undefined
): jest.Mock<TFlowStub> {
  const stub: jest.Mock<TFlowStub> = jest.fn(answer);

  Object.defineProperty(instance, key, { configurable: true, value: stub });

  return stub;
}
