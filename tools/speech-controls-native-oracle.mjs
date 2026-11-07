// Actual pinned contracts only. No model, microphone, speech worker or provider runs.
import fs from 'node:fs';import crypto from 'node:crypto';import {createRequire} from 'node:module';import {pathToFileURL} from 'node:url';
if(!fs.existsSync('/run/.containerenv'))throw Error('Official Android sandbox required');
const [native,out]=process.argv.slice(2);
const sourceHash=crypto.createHash('sha256').update(fs.readFileSync(native+'/packages/contracts/src/speech.ts')).digest('hex');
if(sourceHash!=='2d414456f87199db5bcd61eb9a2e32f476806d0716af3d03fbbb1d9f0f2e4915')throw Error('Changed native speech contracts');
const require=createRequire(native+'/package.json');
const Schema=await import(pathToFileURL(require.resolve('effect/Schema')).href);
const C=await import(pathToFileURL(native+'/packages/contracts/src/speech.ts').href);
const schemas={model:C.EnvironmentSpeechModelRequest,acceleration:C.EnvironmentSpeechAccelerationRequest,timeout:C.EnvironmentSpeechModelUnloadTimeoutRequest,postprocess:C.EnvironmentSpeechPostProcessingRequest};
const cases=[];
function add(family,input){try{const decoded=Schema.decodeUnknownSync(schemas[family])(input);cases.push({family,input,valid:true,encoded:Schema.encodeSync(schemas[family])(decoded)});}catch{cases.push({family,input,valid:false});}}
for(const value of ['', 'x',' x ', 'x'.repeat(160),'x'.repeat(161),'😀'.repeat(80),'😀'.repeat(81),null,1])add('model',{modelId:value,extra:'discarded'});
add('model',{});
for(const value of ['auto','cpu','gpu:x','gpu:','gpu:\n','gpu:\r','gpu:\u2028','gpu:\u2029','gpu:x\n','gpu:\0','gpu:'+'😀'.repeat(248),'gpu:'+'😀'.repeat(249),'GPU:x',' cpu ',null,1])add('acceleration',{acceleration:value});
for(const value of ['never','immediately','min_2','min_5','min_10','min_15','hour_1','minute_2','min_1',' never ',null,1])add('timeout',{timeout:value});
const base={options:{speechCustomWords:[],speechPostProcessingPrompt:{mode:'default',customInstructions:''}},transcript:'Synthetic transcript',draft:{text:'Synthetic draft',selection:{start:0,end:4}}};
const clone=()=>structuredClone(base);
add('postprocess',base);add('postprocess',{...base,extra:'discarded'});
for(const field of ['options','transcript','draft']){const v=clone();delete v[field];add('postprocess',v);add('postprocess',{...base,[field]:null});}
for(const text of ['', '😀'.repeat(50_000),'😀'.repeat(50_001)])for(const field of ['transcript','draftText']){const v=clone();if(field==='transcript')v.transcript=text;else v.draft.text=text;add('postprocess',v);}
for(const value of [-2,0,1.5,9007199254740993,-9007199254740993,1e100,null,'4']){const v=clone();v.draft.selection={start:value,end:value};add('postprocess',v);}
for(const raw of ['9007199254740993','-9007199254740993','-0']){const inputJson=JSON.stringify(base).replace('"start":0','"start":'+raw);add('postprocess',JSON.parse(inputJson));cases.at(-1).inputJson=inputJson;}
for(const mode of ['default','custom','other',null]){const v=clone();v.options.speechPostProcessingPrompt.mode=mode;add('postprocess',v);}
for(const text of ['😀'.repeat(5000),'😀'.repeat(5001),null]){const v=clone();v.options.speechPostProcessingPrompt.customInstructions=text;add('postprocess',v);}
for(const cue of ['', '😀'.repeat(25),'😀'.repeat(26),null]){const v=clone();v.options.speechCorrectionWord=cue;add('postprocess',v);}
for(const terms of [['café','cafe\u0301'],['ΟΣ','ος'],['ΟΣ','οσ'],['\ufeff','x'],['\u0085','x']]){const v=clone();v.options.speechCustomWords=terms.map(term=>({term,aliases:[],extra:'discarded'}));add('postprocess',v);}
fs.writeFileSync(out,JSON.stringify({revision:'83a3674ffd84ffb33de768210c806cb1ac53fd80',sourceHash,unicode:process.versions.unicode,locale:Intl.DateTimeFormat().resolvedOptions().locale,cases},null,2)+'\n');
console.log(JSON.stringify({cases:cases.length,sourceHash}));
