#!/usr/bin/env node
// Execute the pinned native schema; this is not a handwritten validation oracle.
import fs from 'node:fs';
import crypto from 'node:crypto';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
const [source,output]=process.argv.slice(2);
if(!source||!output)throw Error('usage: generate-reset-credit-fixtures.mjs PINNED_NATIVE_SOURCE OUTPUT');
const revision='32a0531748c44b25072d7495ce0b9bd7e9450a87';
if(JSON.parse(fs.readFileSync(source+'/SOURCE.json','utf8')).revision!==revision)throw Error('Native source revision mismatch');
const require=createRequire('/deps/package.json');
const Schema=await import(pathToFileURL(require.resolve('effect/Schema')).href);
const {ServerProviderResetCredits}=await import(pathToFileURL(source+'/packages/contracts/src/providerUsageLimits.ts').href);
const cases=[];
const inputs=[{availableCount:2},...['synthetic-credit','  synthetic-credit  ','\ufeffcredit\u00a0','日本🙂','credit\nline','\0', 'x'.repeat(4096),'', ' ', '\u2007\ufeff', null,0,1,false,true,[],{},['credit']].map(nextCreditId=>({availableCount:2,nextCreditId})),{availableCount:0,nextCreditId:'zero-credit'},{availableCount:2,nextCreditId:'credit',nextExpiresAt:'2026-11-06T23:00:00Z'}];
for(const input of inputs){try{cases.push({input,ok:true,value:Schema.decodeUnknownSync(ServerProviderResetCredits)(input)})}catch{cases.push({input,ok:false})}}
const files=['packages/contracts/src/providerUsageLimits.ts','packages/contracts/src/baseSchemas.ts'];
const hashes=Object.fromEntries(files.map(file=>[file,crypto.createHash('sha256').update(fs.readFileSync(source+'/'+file)).digest('hex')]));
fs.writeFileSync(output,JSON.stringify({revision,effectVersion:'4.1.2',sources:hashes,cases},null,2)+'\n');
console.log('NATIVE_RESET_CREDIT_SCHEMA_PASS: '+cases.length+' exact native cases');
