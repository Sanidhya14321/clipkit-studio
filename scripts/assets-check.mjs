#!/usr/bin/env node
import {spawnSync} from 'node:child_process';
const result=spawnSync(process.execPath,['--import',import.meta.resolve('tsx'),new URL('./assets-check.ts',import.meta.url).pathname,...process.argv.slice(2)],{stdio:'inherit'});
if(result.error)console.error(result.error.message);process.exit(result.status??1);
