import type { JsonValue } from "../generated/types";
export const object = (
  value: JsonValue | undefined,
): Record<string, JsonValue> =>
  value && typeof value === "object" && !Array.isArray(value) ? value : {};
export const text = (value: JsonValue | undefined, fallback = ""): string =>
  typeof value === "string" ? value : fallback;
export const list = (value: JsonValue | undefined): JsonValue[] =>
  Array.isArray(value) ? value : [];
