import { XrfApplicationError } from "@xrf/errors";
import { Maybe, Nullable } from "@xrf/types";
import { createContext, ReactElement, ReactNode, useContext } from "react";

import { IToolDescriptor } from "@/core/routing/tool";

const CurrentToolContext = createContext<Maybe<IToolDescriptor>>(undefined);

export function useCurrentTool(): Nullable<IToolDescriptor> {
  const tool: Maybe<IToolDescriptor> = useContext(CurrentToolContext);

  if (tool === undefined) {
    throw new XrfApplicationError("Current tool is not available.");
  }

  return tool;
}

interface ICurrentToolProviderProps {
  tool: Nullable<IToolDescriptor>;
  children: ReactNode;
}

export function CurrentToolProvider({ tool, children }: ICurrentToolProviderProps): ReactElement {
  return <CurrentToolContext.Provider value={tool}>{children}</CurrentToolContext.Provider>;
}
