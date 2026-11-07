// Execute actual pinned schemas and whole shared framing functions, never native tests.
import fs from 'node:fs';import {createRequire} from 'node:module';import {pathToFileURL} from 'node:url';
if(!fs.existsSync('/run/.containerenv'))throw Error('Private sandbox required');
const native=process.argv[2],out=process.argv[3];const require=createRequire(native+'/package.json');const Schema=await import(pathToFileURL(require.resolve('effect/Schema')).href);
const contracts=await import(pathToFileURL(native+'/packages/contracts/src/speech.ts').href);const reference=await import(pathToFileURL(native+'/packages/contracts/src/secretReference.ts').href);const shared=await import(pathToFileURL(native+'/packages/shared/src/speech.ts').href);
const base={speechLanguage:'auto',speechTranslateToEnglish:false,speechCustomWords:[],speechRemoveFillerWords:true};const cases=[];
function add(family,value){const schema=family==='options'?contracts.SpeechTranscriptionOptions:family==='key'?contracts.CloudSpeechApiKey:family==='reference'?reference.SecretReference:contracts.CloudSpeechModelId;try{const decoded=Schema.decodeUnknownSync(schema)(value);const encoded=Schema.encodeSync(schema)(decoded);const item={family,input:value,valid:true,encoded};if(family==='options'){item.json=Schema.encodeSync(Schema.fromJsonString(schema))(decoded);item.framing=Buffer.from(shared.encodeSpeechPcmRequest(new Uint8Array([0,0,0,0]),decoded)).toString('hex');}cases.push(item);}catch{cases.push({family,input:value,valid:false});}}
add('options',base);add('options',{...base,extra:'discarded'});
for(const k of Object.keys(base)){const v={...base};delete v[k];add('options',v);add('options',{...base,[k]:null});}
for(const value of ['a','en','😀','😀'.repeat(8),'😀'.repeat(9),' x '])add('options',{...base,speechLanguage:value});
for(const value of ['', 'x'.repeat(50),'😀'.repeat(25),'😀'.repeat(26),null])add('options',{...base,speechCorrectionWord:value});
for(const terms of [['café','cafe\u0301'],['ΟΣ','ος'],['ΟΣ','οσ'],['İ','i\u0307'],['\ufeff','x'],['\u0085','x'],['Ａ','ａ'],['K','k'],['é','É'],['😀','😀'],['a',' A ']])add('options',{...base,speechCustomWords:terms.map(term=>({term,aliases:[]}))});
add('options',{...base,speechCustomWords:[{term:'Project',aliases:['project']}]});add('options',{...base,speechSilenceFiltering:false});
for(const value of ['  private-key  ','\ufeffprivate-key\ufeff','https://not-a-key','a+b://no','1://allowed','x y','é','x'.repeat(4096),'x'.repeat(4097),'','\u0085x'])add('key',value);
for(const value of ['author/model','OPENROUTER/model','a/m','author/model:nitro','author/m.o_d-el','é/model','a/b/c','a/.x','-a/m','a/m-','a/b\n','a/b\r','a/b\r\n','a/b\u2028','a/b\u2029'])add('model',value);
for(const value of [{provider:' 1Password ',reference:' op://vault/item/password '},{provider:'a',reference:'x\ny'},{provider:'a',reference:'\u0085'},{provider:'é',reference:'x'}])add('reference',value);
fs.writeFileSync(out,JSON.stringify({revision:'83a3674ffd84ffb33de768210c806cb1ac53fd80',node:process.versions,locale:Intl.DateTimeFormat().resolvedOptions().locale,cases},null,2)+'\n');console.log(JSON.stringify({cases:cases.length,unicode:process.versions.unicode,locale:Intl.DateTimeFormat().resolvedOptions().locale}));
