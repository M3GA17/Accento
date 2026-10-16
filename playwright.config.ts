import {defineConfig} from '@playwright/test';
export default defineConfig({testDir:'tests/ui',fullyParallel:true,use:{baseURL:'http://127.0.0.1:1421',viewport:{width:1040,height:780}},webServer:{command:'npm run dev',url:'http://127.0.0.1:1421',reuseExistingServer:true},reporter:'list'});
