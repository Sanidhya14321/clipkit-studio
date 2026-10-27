const mode=process.argv[2]??'verify';if(!['verify','build'].includes(mode))throw Error('Use compatibility.ts verify or build');await import('../tests/integration/'+(mode==='build'?'compatibility-build.ts':'compatibility.ts'));

export {};
