// Run only in the private Node fixture. Every original module is complete and
// hash-admitted; the checkout/snapshot is the exact nightly 4340 revision.
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import {createHash} from 'node:crypto';
import {createRequire, stripTypeScriptTypes} from 'node:module';
import {pathToFileURL} from 'node:url';
const sourceRoot=process.argv[2]??'/original';
const output=process.argv[3]??'/out/provider-auth-remove-method-original-native-4340.json';
const bridgeOutput=process.argv[4]??'/out/provider-auth-remove-method-native-4340.json';
const fixtureRoot=new URL('../tests/fixtures/',import.meta.url);
const provenance=JSON.parse(fs.readFileSync(new URL('provider-auth-remove-method-4340-source.json',fixtureRoot),'utf8'));
const inputs=JSON.parse(fs.readFileSync(new URL('provider-auth-remove-method-4340-inputs.json',fixtureRoot),'utf8'));
if(provenance.revision!==inputs.revision)throw Error('Native source revision mismatch');
const require=createRequire('/deps/package.json');
const modules=new Map(),synthetics=new Map();
async function dependency(specifier){
 if(synthetics.has(specifier))return synthetics.get(specifier);
 const values=await import(pathToFileURL(require.resolve(specifier)).href);
 const module=new vm.SyntheticModule(Object.keys(values),function(){for(const [key,value]of Object.entries(values))this.setExport(key,value);});
 synthetics.set(specifier,module);return module;
}
function original(relative){
 if(modules.has(relative))return modules.get(relative);
 const expected=provenance.sourceFiles[relative];
 if(!expected||!relative.startsWith('packages/contracts/src/')||relative.includes('..'))throw Error('Unadmitted original source');
 const filename=path.join(sourceRoot,relative),info=fs.lstatSync(filename);
 if(!info.isFile()||info.isSymbolicLink())throw Error('Original module must be a regular file');
 const source=fs.readFileSync(filename,'utf8');
 if(createHash('sha256').update(source).digest('hex')!==expected)throw Error('Original module hash mismatch');
 const module=new vm.SourceTextModule(stripTypeScriptTypes(source,{mode:'strip'}),{identifier:relative});
 modules.set(relative,module);return module;
}
const provider=original('packages/contracts/src/providerSetup.ts');
await provider.link((specifier,reference)=>specifier.startsWith('effect/')
 ?dependency(specifier)
 :original(path.posix.normalize(path.posix.join(path.posix.dirname(reference.identifier),specifier))));
await provider.evaluate();
const Schema=await import(pathToFileURL(require.resolve('effect/Schema')).href);
const decode=Schema.decodeUnknownSync(provider.namespace.ProviderAuthRemoveMethodInput);
const cases=inputs.cases.map(entry=>{
 const input=entry.rawInput===undefined?entry.input:JSON.parse(entry.rawInput);
 try{
  return {...entry,input,ok:true,value:decode(input)};
 }
 catch{return {...entry,input,ok:false};}
});
const decodeStart=Schema.decodeUnknownSync(provider.namespace.ProviderAuthStartInput);
const authStartCases=inputs.authStartCases.map(entry=>{
 try{return {...entry,ok:true,value:decodeStart(entry.input)};}
 catch{return {...entry,ok:false};}
});
const native={complete:true,revision:provenance.revision,wholeOriginalSchemaModule:true,sourceFiles:Object.fromEntries([...modules.keys()].map(name=>[name,provenance.sourceFiles[name]])),cases,authStartCases,qualification:'Whole original auth remove/start schemas with focused inputs only. Native isolated-surrogate outcomes are preserved unchanged; generic Rust JSON ingress cannot represent them. No whole-string equivalence, provider authentication, saved account, credential store, network or mutation executed.'};
const originalText=JSON.stringify(native,null,2)+'\n';
fs.writeFileSync(output,originalText,{flag:'wx',mode:0o400});
const exclusions=cases.filter(entry=>entry.rustJsonRepresentable===false).map(entry=>({name:entry.name,rawInput:entry.rawInput,reason:'generic_rust_json_isolated_surrogate'}));
const bridge={...native,cases:cases.filter(entry=>entry.rustJsonRepresentable!==false),nativeResultSha256:createHash('sha256').update(originalText).digest('hex'),nativeResultFile:path.basename(output),exclusions,qualification:'Rust-readable scalar bridge only. The separate whole native result is retained unchanged with this exact SHA256. Two isolated-surrogate cases are excluded for generic Rust JSON ingress, never converted into fabricated native rejections.'};
fs.writeFileSync(bridgeOutput,JSON.stringify(bridge,null,2)+'\n',{flag:'wx',mode:0o400});
console.log('NATIVE_PROVIDER_AUTH_REMOVE_INPUT_PASS '+cases.length+' START '+authStartCases.length);
