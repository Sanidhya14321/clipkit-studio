import {z} from 'zod';
const ScalarRule=z.object({type:z.enum(['string','number','integer','boolean']),enum:z.array(z.union([z.string(),z.number(),z.boolean()])).min(1).optional(),minimum:z.number().optional(),maximum:z.number().optional(),minLength:z.number().int().nonnegative().optional(),maxLength:z.number().int().nonnegative().optional(),pattern:z.string().optional(),default:z.unknown().optional(),description:z.string().optional()});
/** Explicit nested schema prevents costly model retries with Zod-like or incomplete JSON. */
export const ScenePropsSchema=z.object({type:z.literal('object'),properties:z.record(z.string(),ScalarRule),required:z.array(z.string()).optional(),additionalProperties:z.literal(false),description:z.string().optional()});
