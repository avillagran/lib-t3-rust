#!/usr/bin/env node
// Execute the exact native union, including transform order and excess properties.
import fs from 'node:fs';
import crypto from 'node:crypto';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
const [source,output]=process.argv.slice(2);
const revision='32a0531748c44b25072d7495ce0b9bd7e9450a87';
if(!source||!output||JSON.parse(fs.readFileSync(source+'/SOURCE.json','utf8')).revision!==revision)throw Error('Pinned native input required');
const provenance=JSON.parse(fs.readFileSync(source+'/SOURCE.json','utf8'));
for(const [file,digest] of Object.entries(provenance.files)){
 if(crypto.createHash('sha256').update(fs.readFileSync(source+'/'+file)).digest('hex')!==digest)throw Error('Changed native input: '+file);
}
const require=createRequire('/deps/package.json');
const Schema=await import(pathToFileURL(require.resolve('effect/Schema')).href);
const {ProviderConsumeResetCreditInput}=await import(pathToFileURL(source+'/packages/contracts/src/providerUsageLimits.ts').href);
const target={sourceId:'synthetic-source',accountId:'account',creditId:'credit'};
const inputs=[null,false,[],{}, {instanceId:'codex'},{instanceId:' \ufeffCodex_1-2\u00a0 '},{instanceId:'codex',...target,extra:'ignored'},...['',' ','1codex','a.b','a'.repeat(64),'a'.repeat(65),null,42,false,{},[]].flatMap(instanceId=>[{instanceId},{instanceId,...target}]),target,...['',' ','a:b\nc','日本🙂',null,7,false,{},[]].map(sourceId=>({...target,sourceId})),...['accountId','creditId'].flatMap(field=>['',' ','\ufeff\u2007',' \ufeffvalue\u00a0 ','日本🙂','a\nb','\0','x'.repeat(4096),null,7,false,[],{}].map(value=>({...target,[field]:value}))),{...target,extra:{nested:true}}, {instanceId:{bad:true},...target,ignored:true}];
const cases=inputs.map(input=>{try{return {input,ok:true,value:Schema.decodeUnknownSync(ProviderConsumeResetCreditInput)(input)}}catch{return {input,ok:false}}});
const files=['packages/contracts/src/providerUsageLimits.ts','packages/contracts/src/providerInstance.ts','packages/contracts/src/usageLimitSourceId.ts','packages/contracts/src/baseSchemas.ts'];
const sources=Object.fromEntries(files.map(file=>[file,crypto.createHash('sha256').update(fs.readFileSync(source+'/'+file)).digest('hex')]));
fs.writeFileSync(output,JSON.stringify({revision,effectVersion:'4.1.2',sources,cases},null,2)+'\n');
console.log('NATIVE_RESET_CREDIT_INPUT_PASS: '+cases.length+' exact union cases');
