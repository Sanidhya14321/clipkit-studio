import {defineConfig} from '@playwright/test';
import {existsSync} from 'node:fs';
const executablePath=process.env.CHROME_BIN??['/usr/bin/google-chrome','/usr/bin/chromium','/opt/google/chrome/chrome'].find(existsSync);
export default defineConfig({
  testDir:'./tests/e2e',fullyParallel:false,workers:1,timeout:60000,
  use:{baseURL:'http://127.0.0.1:4320',viewport:{width:1440,height:1000},headless:true,launchOptions:{...(executablePath?{executablePath}:{}),args:['--no-sandbox']},screenshot:'only-on-failure',trace:'retain-on-failure'},
  reporter:[['list'],['html',{open:'never'}]],
  webServer:{command:'pnpm ck studio --no-vite --debug-harness --port 4320 --root .clipkit/e2e-v3-workspace',url:'http://127.0.0.1:4320/api/bootstrap',reuseExistingServer:false,timeout:30000,env:{FISH_API_KEY:''}},
});
