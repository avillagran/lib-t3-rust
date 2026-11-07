import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import {pathToFileURL} from "node:url";
if(process.env.ANDROID_SANDBOX!=="1"||!fs.existsSync("/run/.containerenv"))throw Error("Run this oracle through tools/android-sandbox");
const [installation,output]=process.argv.slice(2);
if(!installation||!output)throw Error("Usage: node generate-semu-fixtures.mjs INSTALLATION OUTPUT");
const packageRoot=path.join(installation,"node_modules/expo-device-hub");
const metadata=JSON.parse(fs.readFileSync(path.join(packageRoot,"package.json"),"utf8"));
if(metadata.name!=="expo-device-hub"||metadata.version!=="0.19.1")throw Error("The pinned expo-device-hub 0.19.1 installation is required");
const relative="vendor/serve-emu/dist/shared/frame-meta.js",modulePath=path.join(packageRoot,relative);
const source=fs.readFileSync(modulePath);const native=await import(pathToFileURL(modulePath).href);
const cases=[];
function add(id,packet,legacyEmpty=false){const value=native.parseFramePacket(packet);cases.push({id,packet:Array.from(packet),native:{data:Array.from(value.data),isKey:value.isKey,timestamp:value.timestamp,serverTsMs:value.serverTsMs},legacyV1Empty:legacyEmpty?{isKey:(packet[5]&1)!==0}:null})}
const points=[0n,42n,9007199254740991n,9007199254740992n,18446744073709551615n];
for(const version of [1,2])for(const length of [0,1,4,5,32,256])for(const flags of [0,1,3,128,255])for(const pts of points){
 const payload=Uint8Array.from({length},(_,index)=>index<4?[0,0,0,1][index]:(index*37+flags)%256);
 const header=version===1?16:native.FRAME_META_HEADER_BYTES;const packet=new Uint8Array(header+payload.length);
 if(version===2)native.writeFrameMetaHeader(packet,{isKey:(flags&1)!==0,pts,serverTsMs:1777777777000.125+cases.length/8});
 else {const view=new DataView(packet.buffer);view.setUint32(0,native.FRAME_META_MAGIC,false);view.setUint8(4,1);view.setBigUint64(8,pts,false)}
 packet[5]=flags;packet.set(payload,header);add(`v${version}/bytes${length}/flags${flags}/pts${pts}`,packet,version===1&&length===0);
}
const complete=new Uint8Array(32);native.writeFrameMetaHeader(complete,{isKey:true,pts:42n,serverTsMs:1777777777000});complete.set([0,0,0,1,0x65,0x88,0x84,0x21],24);
for(let length=0;length<=32;length++)add(`v2/truncated/${length}`,complete.slice(0,length));
for(const version of [0,3,255]){const packet=complete.slice();packet[4]=version;add(`unknown-version/${version}`,packet)}
const wrongMagic=complete.slice();wrongMagic[3]=0x58;add("wrong-magic",wrongMagic);add("raw-annex-b",Uint8Array.of(0,0,0,1,0x65,0x88,0x84,0x21));
const lock=JSON.parse(fs.readFileSync(path.join(installation,"package-lock.json"),"utf8"));
const fixture={source:{package:"expo-device-hub",version:"0.19.1",module:relative,sha256:crypto.createHash("sha256").update(source).digest("hex"),integrity:lock.packages["node_modules/expo-device-hub"].integrity,execution:"Whole installed native writeFrameMetaHeader and parseFramePacket exports"},cases};
fs.mkdirSync(path.dirname(output),{recursive:true});if(fs.existsSync(output)&&!fs.lstatSync(output).isFile())throw Error("Output must be a regular file");fs.writeFileSync(output,JSON.stringify(fixture,null,2)+"\n");
console.log(JSON.stringify({cases:cases.length,recognized:cases.filter(row=>typeof row.native.isKey==="boolean").length,legacyEmpty:cases.filter(row=>row.legacyV1Empty!==null).length,sourceSha256:fixture.source.sha256}));
