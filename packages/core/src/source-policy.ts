import {z} from 'zod';
export const LicenseClassSchema=z.enum(['permissive','attribution-required','copyleft','commercial-restricted','account-or-key-required','proprietary-reference-only','unknown']);
export const LicenseModeSchema=z.enum(['commercial-safe','personal','any']);
export type LicenseClass=z.infer<typeof LicenseClassSchema>;
export const HubMetadata={sourceUrl:z.string().max(500).optional(),sourceId:z.string().optional(),sourceCommit:z.string().optional(),spdx:z.string().optional(),licenseClass:LicenseClassSchema.optional(),licensePath:z.string().optional(),licenseHash:z.string().optional(),determinism:z.enum(['native','wrappable','reference-only']).optional(),grade:z.enum(['A','B','C']).optional(),scores:z.record(z.string(),z.number().min(0).max(100)).optional(),collections:z.array(z.string()).max(12).optional(),attributionText:z.string().max(1800).optional(),vettedAt:z.string().optional()};
export const collections=['Starter','Text','Backgrounds','Transitions','Product demo','Social','Cinematic','Audio','Look packs'] as const;
export function licenseAllowed(kind:LicenseClass|undefined,mode:string){return mode==='any'?true:mode==='personal'?!['unknown','account-or-key-required','proprietary-reference-only','copyleft'].includes(kind??'unknown'):['permissive','attribution-required'].includes(kind??'unknown')}
export function projectLicenseMode(p:{licenseMode?:string;brief?:any}){return p.licenseMode??p.brief?.fields?.licenseMode?.value??(p.brief?'commercial-safe':'any')}
export function assertLicense(asset:{id?:string;licenseClass?:LicenseClass},mode:string){if(!licenseAllowed(asset.licenseClass,mode))throw Error(`LICENSE_MODE_BLOCKED: ${asset.id??'asset'} is ${asset.licenseClass??'unknown'} in ${mode} mode; inspect its original license evidence.`)}
