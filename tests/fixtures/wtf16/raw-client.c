/* Real exported C ABI and JavaScript UTF-16 wire round trips. */
#include "t3_protocol.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
static char *raw(void *handle,const char *request){return t3_client_call_raw(handle,(const uint8_t *)request,strlen(request));}
int main(int argc,char **argv){
 assert(argc==2);assert(strncmp(argv[1],"http://127.0.0.1:",17)==0);void *handle=t3_client_new();char request[2048];
 snprintf(request,sizeof(request),"{\"op\":\"connectBearer\",\"httpBaseUrl\":\"%s\",\"token\":\"synthetic-wtf16\"}",argv[1]);
 char *answer=t3_client_call(handle,(const uint8_t *)request,strlen(request));assert(strstr(answer,"\"ok\":true"));t3_string_free(answer);
 const char *queries[]={"ab\\ud800","ab\\udc00","ab\\ud83d\\ude00"};
 for(size_t i=0;i<3;i++){
  snprintf(request,sizeof(request),"{\"op\":\"searchThreadsUtf16\",\"query\":\"%s\",\"options\":{}}",queries[i]);answer=raw(handle,request);
  assert(strstr(answer,"\"ok\":true"));assert(strstr(answer,"\"snippet\":"));assert(strstr(answer,"\\ud83d"));
  if(i<2)assert(strstr(answer,i==0?"ab\\ud800":"ab\\udc00"));else assert(strstr(answer,"😀"));t3_string_free(answer);
 }
 char clipped[256];memset(clipped,'a',199);strcpy(clipped+199,"\\ud83d");
 snprintf(request,sizeof(request),"{\"op\":\"searchThreadsUtf16\",\"query\":\"%s\",\"options\":{}}",clipped);answer=raw(handle,request);assert(strstr(answer,"\"ok\":true"));assert(strstr(answer,"55357"));t3_string_free(answer);
 answer=raw(handle,"{\"op\":\"rpc\",\"tag\":\"fixture.fail\",\"payload\":{\"query\":\"ab\\udc00\"}}");assert(strstr(answer,"rpc_rejected"));assert(strstr(answer,"ab\\udc00"));t3_string_free(answer);
 answer=raw(handle,"{\"op\":\"rpc\",\"tag\":\"x\",\"payload\":\"\\uxxxx\"}");assert(strstr(answer,"malformed"));t3_string_free(answer);
 t3_client_free(handle);puts("WTF16_C_ABI_PASS high low pair clipped nativeSnippet failure malformed");return 0;
}
