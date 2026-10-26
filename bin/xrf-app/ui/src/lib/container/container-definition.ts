import { ContainerConfig } from "@wirestate/core";

/** Container bindings and options whose parent is supplied by the owning scope. */
export type ContainerDefinition = Omit<ContainerConfig, "parent">;
