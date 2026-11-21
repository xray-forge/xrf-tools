import { ContainerConfig } from "@wirestate/core";

/** Container bindings and options whose parent is supplied by the owning scope. */
export type IContainerDefinition = Omit<ContainerConfig, "parent">;
