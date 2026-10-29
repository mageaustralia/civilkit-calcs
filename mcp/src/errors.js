// Errors shared by the engine and the tools (kept apart so the tool list loads without the generated wasm).

/** A problem with the input that the model can act on; answered as a tool error. */
export class EngineError extends Error {}
