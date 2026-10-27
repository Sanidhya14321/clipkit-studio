import {ProjectSchema,type Project} from './index';
/** Additive v1 migration: optional pipeline fields are now retained and validated.
 * No inference/default persistence: old snapshot and journal pointers remain byte-compatible.
 * Mode/plan are initialized only by an explicit pipeline plan operation. */
export function migrateProject(raw:unknown):Project{return ProjectSchema.parse(raw)}
