import { createContext, ReactElement, ReactNode, useContext } from "react";

import { XrfApplicationError } from "@/core/error/lib";
import { IApplicationDescriptor } from "@/core/routing/application";
import { Maybe, Nullable } from "@/lib/types/general";

const CurrentApplicationContext = createContext<Maybe<IApplicationDescriptor>>(undefined);

export function useCurrentApplication(): Nullable<IApplicationDescriptor> {
  const application: Maybe<IApplicationDescriptor> = useContext(CurrentApplicationContext);

  if (application === undefined) {
    throw new XrfApplicationError("Current application is not available.");
  }

  return application;
}

interface ICurrentApplicationProviderProps {
  application: Nullable<IApplicationDescriptor>;
  children: ReactNode;
}

export function CurrentApplicationProvider({ application, children }: ICurrentApplicationProviderProps): ReactElement {
  return <CurrentApplicationContext.Provider value={application}>{children}</CurrentApplicationContext.Provider>;
}
