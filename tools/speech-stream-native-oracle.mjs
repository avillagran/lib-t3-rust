// Execute the entire pinned client stream producer with an inert socket/clock.
import fs from 'node:fs';import crypto from 'node:crypto';import vm from 'node:vm';import {stripTypeScriptTypes,createRequire} from 'node:module';import {pathToFileURL} from 'node:url';
if(!fs.existsSync('/run/.containerenv'))throw Error('Official sandbox required');
const [native,out]=process.argv.slice(2),file='packages/client-runtime/src/voice-input/stream.ts';
const source=fs.readFileSync(native+'/'+file,'utf8'),hash=crypto.createHash('sha256').update(source).digest('hex');
if(hash!=='1b0c169333e3b8d3ece2c677d11190c098125f8509c3528aa9be87a6f54c59b9')throw Error('Changed original client');
const contract=fs.readFileSync(native+'/packages/contracts/src/speech.ts');if(crypto.createHash('sha256').update(contract).digest('hex')!=='2d414456f87199db5bcd61eb9a2e32f476806d0716af3d03fbbb1d9f0f2e4915')throw Error('Changed contracts');
const require=createRequire(native+'/package.json'),Schema=await import(pathToFileURL(require.resolve('effect/Schema')).href),C=await import(pathToFileURL(native+'/packages/contracts/src/speech.ts').href);
const options=C.DEFAULT_SPEECH_TRANSCRIPTION_OPTIONS;
const scripts=[];const add=(name,steps)=>scripts.push({name,steps});
const open={op:'open'},ready={op:'message',value:{type:'ready'}},finish={op:'finish'},feed=(count,value=0.125)=>({op:'feed',count,value}),update=(revision=0,text=null)=>({op:'message',value:{type:'update',revision,text}}),done={op:'message',value:{type:'finished',text:'Synthetic final'}};
add('finish-before-ready',[finish,open,ready,done,feed(2)]);
add('queued-window-and-tail-merge',[feed(1),feed(2),feed(40000),open,ready,finish,update(0,{committed:'a',tentative:'b'}),update(0),update(1),done]);
add('exact-outstanding-bound',[feed(4800000),open,ready,update(0),feed(16000),feed(1)]);
add('over-outstanding-bound',[feed(4800001),open,ready]);
add('empty-feeds',[open,ready,feed(0),finish,feed(3),done]);
add('nonfinite-wire',[open,ready,{op:'feed',count:3,pattern:'nan'},update(),finish,done]);
for(const [name,steps]of [
 ['duplicate-ready',[open,ready,ready]],['update-before-ready',[open,update()]],['decreasing-revision',[open,ready,feed(32000),update(7),update(6)]],
 ['negative-revision',[open,ready,feed(1),update(-2)]],['equal-negative-one',[open,ready,feed(32000),update(-1),update(-1),finish,done]],
 ['large-integer-revision',[open,ready,feed(1),update(1e30),finish,done]],['safe-integer-max',[open,ready,feed(1),update(9007199254740991),finish,done]],['unsafe-integer-first',[open,ready,feed(1),update(9007199254740992),finish,done]],['fractional-revision',[open,ready,feed(1),update(0.5)]],
 ['missing-text',[open,ready,feed(1),{op:'message',value:{type:'update',revision:0}}]],['malformed-text',[open,ready,feed(1),update(0,{committed:7,tentative:''})]],
 ['early-finished',[open,ready,done]],['invalid-json',[open,{op:'raw',value:'{' }]],['unknown-event',[open,{op:'message',value:{type:'future'}}]],
 ['remote-error',[open,{op:'message',value:{type:'error',message:'Synthetic remote error'}}]],['closed',[open,{op:'socket-close'}]],['socket-error',[open,{op:'socket-error'}]],
 ['cancelled-opening',[{op:'cancel'},open]],['cancelled-ready',[open,ready,{op:'cancel'},feed(1)]],
 ['opening-timeout',[{op:'tick',ms:119999},{op:'tick',ms:1},open]],['idle-ready-no-watchdog',[open,ready,{op:'tick',ms:500000},finish,done]],
 ['ack-rearms-other-flight',[open,ready,feed(32000),{op:'tick',ms:119000},update(),{op:'tick',ms:119999},{op:'tick',ms:1}]],
 ['finished-watchdog',[open,ready,finish,{op:'tick',ms:120000}]],
 ['consent-before-open',[{op:'consent',value:false},open]],['consent-before-ready',[open,{op:'consent',value:false},ready]],
 ['consent-before-feed',[open,ready,{op:'consent',value:false},feed(1)]],['consent-before-update',[open,ready,feed(1),{op:'consent',value:false},update()]],
 ['unknown-fields',[open,{op:'message',value:{type:'ready',future:true}},feed(1),{op:'message',value:{type:'update',revision:0,text:null,future:'ignored'}},finish,done]],
])add(name,steps);
let seed=4314;const random=()=>{seed=(Math.imul(seed,1664525)+1013904223)>>>0;return seed;};
for(let index=0;index<32;index++){
 const steps=[open,ready];let revision=0;
 for(let i=0;i<24;i++){if(random()%3===0)steps.push(update(revision++,(random()%2)?{committed:'Synthetic '+i,tentative:'next'}:null));else steps.push(feed(random()%48001,(random()%17-8)/16));}
 steps.push(finish,update(revision++),update(revision++),update(revision++),done);add('seeded-'+index,steps);
}
const known=new Set(['Voice transcription was cancelled.','Voice consent was revoked.','Live transcription stopped responding.','Voice recordings are limited to five minutes.','Unexpected speech stream response.','Synthetic remote error','The voice connection closed. Please record again.','Could not connect to live transcription.']);
const normalize=message=>known.has(message)?message:'invalid_response';
const cases=[];
for(const script of scripts){
 let now=0,nextTimer=0,canSend=true,final=null;const timers=new Map(),frames=[],texts=[],errors=[];
 class Socket{listeners=new Map();addEventListener(name,fn){this.listeners.set(name,fn);}send(data){if(typeof data==='string')frames.push({kind:'text',value:JSON.parse(data)});else{const bytes=Buffer.from(data.buffer,data.byteOffset,data.byteLength);frames.push({kind:'audio',bytes:bytes.length,sha256:crypto.createHash('sha256').update(bytes).digest('hex')});}}close(){frames.push({kind:'close'});}emit(name,data){this.listeners.get(name)?.(data);}}
 const socket=new Socket(),controller=new AbortController();
 const context=vm.createContext({AbortController,Uint8Array,Float32Array,Error,JSON,setTimeout:(callback,ms)=>{const id=++nextTimer;timers.set(id,{callback,at:now+ms});return id;},clearTimeout:id=>timers.delete(id)});
 const imports={ '@t3tools/contracts':{SPEECH_STREAM_MAX_CHUNK_BYTES:C.SPEECH_STREAM_MAX_CHUNK_BYTES,SPEECH_STREAM_MAX_QUEUED_BYTES:C.SPEECH_STREAM_MAX_QUEUED_BYTES,SpeechStreamEvent:C.SpeechStreamEvent}, 'effect/Schema':Schema };
 const module=new vm.SourceTextModule(stripTypeScriptTypes(source,{mode:'strip'}),{context});
 await module.link(specifier=>{const values=imports[specifier];if(!values)throw Error('Unexpected import');return new vm.SyntheticModule(Object.keys(values),function(){for(const [k,v]of Object.entries(values))this.setExport(k,v);},{context});});await module.evaluate();
 const stream=module.namespace.openSpeechStream({url:'ws://synthetic.invalid/',options,signal:controller.signal,canSend:()=>canSend,onText:text=>texts.push(text),onError:error=>errors.push(normalize(error.message)),createSocket:()=>socket});
 const output=[];
 for(const step of script.steps){
  switch(step.op){case'open':socket.emit('open');break;case'feed':{const backing=new Float32Array(step.count+11).fill(99);backing.fill(step.pattern==='nan'?NaN:step.value,7,7+step.count);stream.feed(backing.subarray(7,7+step.count));break;}case'message':socket.emit('message',{data:JSON.stringify(step.value)});break;case'raw':socket.emit('message',{data:step.value});break;case'finish':stream.finish().then(text=>{final={ok:true,text};},error=>{final={ok:false,error:normalize(error.message)};});break;case'cancel':stream.cancel();break;case'consent':canSend=step.value;break;case'socket-close':socket.emit('close');break;case'socket-error':socket.emit('error');break;case'tick':{const until=now+step.ms;while(true){const row=[...timers].filter(([,t])=>t.at<=until).sort((a,b)=>a[1].at-b[1].at)[0];if(!row)break;now=row[1].at;timers.delete(row[0]);row[1].callback();}now=until;break;}default:throw Error('Unknown fixture operation');}
  await Promise.resolve();await Promise.resolve();output.push(structuredClone({frames,texts,errors,final,timers:[...timers.values()].map(t=>t.at-now).sort((a,b)=>a-b)}));
 }
 controller.abort();cases.push({...script,output});
}
fs.writeFileSync(out,JSON.stringify({revision:'83a3674ffd84ffb33de768210c806cb1ac53fd80',sourceFile:file,sourceSha256:hash,options,cases},null,2)+'\n');console.log(JSON.stringify({cases:cases.length,steps:cases.reduce((n,c)=>n+c.steps.length,0)}));
