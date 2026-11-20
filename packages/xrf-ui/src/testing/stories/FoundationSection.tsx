import { ReactElement, ReactNode } from "react";

interface IFoundationSectionProps {
  title: string;
  /** What the section's values are for, and the rule they keep. */
  description: string;
  children: ReactNode;
}

/** One block of a foundation page: a heading, the rule it shows, and the specimens. */
export function FoundationSection({ title, description, children }: IFoundationSectionProps): ReactElement {
  return (
    <section className={"flex max-w-reading flex-col gap-3"}>
      <div>
        <h2 className={"text-base font-semibold"}>{title}</h2>
        <p className={"text-sm text-text-secondary"}>{description}</p>
      </div>

      {children}
    </section>
  );
}
