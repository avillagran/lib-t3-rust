// Execute the complete native Devices access resolver with an injected HTTP boundary.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {stripTypeScriptTypes} from 'node:module';
import {pathToFileURL} from 'node:url';
if(process.env.ANDROID_SANDBOX!=='1'||!fs.existsSync('/run/.containerenv'))throw Error('Run through tools/android-sandbox');
const [checkout, dependencies, output] = process.argv.slice(2);
if(!output)throw Error('Usage: CHECKOUT DEPENDENCIES OUTPUT');
const revision='4cb6945dc60ed03f567dc5daafd083e52308e82b';
const sources={};
function source(name){const text=execFileSync('git',['-C',checkout,'show',revision+':'+name],{encoding:'utf8'});sources[name]=crypto.createHash('sha256').update(text).digest('hex');return text;}
const access=source('packages/client-runtime/src/state/deviceHubAccess.ts');
const endpoint=source('packages/client-runtime/src/environment/endpoint.ts');
const hub=source('packages/client-runtime/src/device/hubAccess.ts');
const effect=name=>pathToFileURL(path.join(dependencies,'node_modules/effect/dist',name+'.js')).href;
const Effect=await import(effect('Effect'));
fs.mkdirSync(output,{recursive:true});
const runtime=path.join(output,'runtime');fs.mkdirSync(runtime,{recursive:true});
function write(name,text){const target=path.join(runtime,name+'.mjs');fs.writeFileSync(target,text);return pathToFileURL(target).href;}
const endpointUrl=write('endpoint',stripTypeScriptTypes(endpoint).replace('export * from "@t3tools/shared/advertisedEndpoint";',''));
const hubUrl=write('hub',stripTypeScriptTypes(hub));
const mocks=write('services',`import * as Effect from ${JSON.stringify(effect('Effect'))}; import * as Context from ${JSON.stringify(effect('Context'))}; import * as Redacted from ${JSON.stringify(effect('Redacted'))};
export const RemoteEnvironmentAuthorization=Context.Service('fixture.remote'); export const ManagedRelayDpopSigner=Context.Service('fixture.signer');
export const requests=[]; let failure=false; export const refuse=value=>{failure=value;};
export const executeAuthenticatedEnvironmentHttpRequest=input=>{const route=input.url({webSocketTicketFor:({params})=>'/api/auth/websocket-ticket/'+params.use});const calls=[];const response=input.request({client:{webSocketTicketFor:value=>{calls.push(value);return value;}},headers:{authorization:'Bearer synthetic'}});requests.push({method:input.method,group:input.group,route,timeoutMs:input.timeoutMs,calls,response});return failure?Effect.fail('synthetic refusal'):Effect.succeed({ticket:Redacted.make('synthetic-media-ticket')});};`);
const imports={'effect/Effect':effect('Effect'),'effect/Redacted':effect('Redacted'),'../environment/endpoint.ts':endpointUrl,'../device/hubAccess.ts':hubUrl};
const transformed=stripTypeScriptTypes(access).replace(/(from\s*)(["'])([^"']+)\2/g,(_,prefix,quote,specifier)=>prefix+JSON.stringify(imports[specifier]??mocks));
const native=await import(write('native',transformed));const boundary=await import(mocks);
const cases=[];
for(const authorization of [null,{_tag:'Bearer',token:'synthetic'},{_tag:'DPoP',accessToken:'synthetic'}])for(const failure of [false,true]){
 boundary.requests.length=0;boundary.refuse(failure);
 const input={prepared:{httpBaseUrl:'http://127.0.0.1:1234/prefix?old=1#fragment',httpAuthorization:authorization},hubBasePath:'/api/device-hub'};
 const result=await Effect.runPromise(Effect.result(native.resolveDeviceHubAccess(input)));
 cases.push({authorization:authorization?._tag??'Cookie',failure,result:result._tag==='Success'?{ok:true,value:result.success}:{ok:false},requests:[...boundary.requests]});
}
if(cases.filter(row=>row.requests.length).some(row=>row.requests.length!==1||row.requests[0].route!=='/api/auth/websocket-ticket/media'))throw Error('Unexpected native media purpose');
fs.writeFileSync(path.join(output,'fixtures.json'),JSON.stringify({revision,sources,execution:'Complete native resolver, real Effect 4.1.2; only HTTP transport/service tags injected',cases},null,2)+'\n');
console.log(JSON.stringify({cases:cases.length,requests:cases.reduce((sum,row)=>sum+row.requests.length,0),sources}));
