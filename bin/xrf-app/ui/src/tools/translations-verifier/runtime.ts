import { ContainerDefinition } from "@/lib/container/container-definition";
import { TranslationsVerifierService } from "@/tools/translations-verifier/services/translations-verifier.service";

export { TranslationsVerifierTool as Component } from "./TranslationsVerifierTool";

export const container: ContainerDefinition = { bindings: [TranslationsVerifierService] };
