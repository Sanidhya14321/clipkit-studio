export const stageBudgetMs=(profile?:string)=>['production','creative'].includes(profile??'')?360000:180000;
export const successfulAgentExit=(code:number|null,timedOut:boolean,stderr:string)=>code===0&&!timedOut&&!stderr.includes('not logged in');
