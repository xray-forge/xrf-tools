import { Component, ComponentType, ErrorInfo, ReactNode } from "react";

import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

export interface IErrorBoundaryFallbackProps {
  error: Error;
  /** Re-renders the subtree that threw, without reloading the window. */
  onRetry: () => void;
}

export interface IErrorBoundaryProps {
  children: ReactNode;
  fallback: ComponentType<IErrorBoundaryFallbackProps>;
  /**
   * Clears a caught error whenever it changes.
   *
   * Without it a crash is permanent: the boundary keeps rendering its fallback even after the user has
   * navigated somewhere else, because nothing else ever resets the state.
   */
  resetKey?: string;
  /**
   * Reports a caught crash to whoever wants a durable record of it.
   */
  onCaught?: (error: Error, componentStack: Nullable<string>) => void;
}

interface IErrorBoundaryState {
  error: Nullable<Error>;
}

/**
 * Catches render and lifecycle failures in the subtree below it.
 */
export class ErrorBoundary extends Component<IErrorBoundaryProps, IErrorBoundaryState> {
  public static getDerivedStateFromError(error: Error): IErrorBoundaryState {
    return { error };
  }

  private readonly log: Logger = new Logger(__MODULE_NAME__);

  public constructor(props: IErrorBoundaryProps) {
    super(props);

    this.state = { error: null };
  }

  public componentDidCatch(error: Error, info: ErrorInfo): void {
    // The component stack is the only part that says *where* it broke, and it exists nowhere else.
    this.log.error("Render failed:", error, info.componentStack);

    this.props.onCaught?.(error, info.componentStack ?? null);
  }

  public componentDidUpdate(previous: IErrorBoundaryProps): void {
    if (this.state.error && previous.resetKey !== this.props.resetKey) {
      this.setState({ error: null });
    }
  }

  public render(): ReactNode {
    const { error } = this.state;

    if (error) {
      const Fallback: ComponentType<IErrorBoundaryFallbackProps> = this.props.fallback;

      return <Fallback error={error} onRetry={() => this.setState({ error: null })} />;
    } else {
      return this.props.children;
    }
  }
}
