// A private real JavaScript JSON/WebSocket peer, not a search scoring oracle.
// Transport proof only. Host the original source expression from queries.ts:112.
const http=require('node:http'),crypto=require('node:crypto');
if(!require('node:fs').existsSync('/run/.containerenv')||process.versions.node!=='26.13.0')throw Error('Private pinned Node required');
const sockets=new Set();let requests=0,closed=0,listenerClosed=false,stopping=false,reported=false,retirementTimer;
function finished(){if(stopping&&listenerClosed&&sockets.size===0&&!reported){reported=true;clearTimeout(retirementTimer);console.log(JSON.stringify({requests,closed,remaining:0,terminal:true}));}}
function encode(op,payload){const bytes=Buffer.from(payload);let header;if(bytes.length<126){header=Buffer.from([0x80|op,bytes.length]);}else{header=Buffer.alloc(4);header[0]=0x80|op;header[1]=126;header.writeUInt16BE(bytes.length,2);}return Buffer.concat([header,bytes]);}
const server=http.createServer((request,response)=>{response.writeHead(404);response.end();});
server.on('upgrade',(request,socket)=>{
 if(request.headers.authorization!=='Bearer synthetic-wtf16'){socket.end('HTTP/1.1 401 Unauthorized\r\n\r\n');return;}
 const accept=crypto.createHash('sha1').update(request.headers['sec-websocket-key']+'258EAFA5-E914-47DA-95CA-C5AB0DC85B11').digest('base64');
 socket.write('HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: '+accept+'\r\n\r\n');
 sockets.add(socket);socket.on('error',()=>{});socket.on('close',()=>{sockets.delete(socket);closed++;finished();});let pending=Buffer.alloc(0);
 socket.on('data',data=>{pending=Buffer.concat([pending,data]);while(pending.length>=2){
  const opcode=pending[0]&15;let length=pending[1]&127,offset=2;
  if(!(pending[1]&128)||!(pending[0]&128))throw Error('Fixture expects one complete masked client frame');
  if(length===126){if(pending.length<4)return;length=pending.readUInt16BE(2);offset=4;}
  else if(length===127)throw Error('Fixture message exceeds bounded frame');
  if(length>16384)throw Error('Fixture frame too large');if(pending.length<offset+4+length)return;
  const mask=pending.subarray(offset,offset+4),payload=Buffer.from(pending.subarray(offset+4,offset+4+length));pending=pending.subarray(offset+4+length);for(let i=0;i<payload.length;i++)payload[i]^=mask[i%4];
  if(opcode===8){socket.end(encode(8,payload));return;}if(opcode===9){socket.write(encode(10,payload));continue;}if(opcode===10)continue;if(opcode!==1)throw Error('Unexpected opcode');
  const frame=JSON.parse(payload.toString('utf8'));if(frame._tag==='Ping'){socket.write(encode(1,JSON.stringify({_tag:'Pong'})));continue;}
  if(frame._tag!=='Request'||frame.id!=='1')throw Error('Unexpected request');requests++;
  const query=frame.payload.query;if(typeof query!=='string')throw Error('String query required');
  // These actual native operations intentionally retain isolated UTF-16 units.
  const normalized=query.trim().slice(0,200).trim();
  const units=Array.from({length:normalized.length},(_,i)=>normalized.charCodeAt(i));
  const value={query:normalized,units,snippet:('s'.repeat(239)+'😀').slice(0,240)};
  const exit=frame.tag==='fixture.fail'?{_tag:'Failure',cause:{_tag:'Fail',error:{reason:query}}}:{_tag:'Success',value};
  socket.write(encode(1,JSON.stringify({_tag:'Exit',requestId:frame.id,exit})));
 }});
});
server.listen(0,'127.0.0.1',()=>console.log(JSON.stringify({port:server.address().port,node:process.versions.node})));
process.stdin.resume();process.stdin.on('end',()=>{
 stopping=true;
 retirementTimer=setTimeout(()=>{process.exitCode=1;for(const socket of sockets)socket.destroy();},2000);
 server.close(()=>{listenerClosed=true;finished();});finished();
});
