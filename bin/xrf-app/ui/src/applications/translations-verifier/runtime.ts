import { TranslationsVerifierService } from "@/applications/translations-verifier/services/verifier";
import { ContainerDefinition } from "@/lib/container/container-definition";

export { TranslationsVerifierApplication as Component } from "./TranslationsVerifierApplication";

export const container: ContainerDefinition = { bindings: [TranslationsVerifierService] };
