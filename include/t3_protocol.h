#ifndef T3_PROTOCOL_H
#define T3_PROTOCOL_H
#define T3_PROTOCOL_ABI_VERSION 2
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Run blocking calls on a worker thread. Host owns login and secure storage.
 * Calls may be concurrent. Never free a handle concurrently with a call.
 * JSON: connectBearer, connectRelay, request, rpc, disconnect,
 * subscribe, pollSubscription, cancelSubscription.
 * Result: {"ok":true,"data":...} or {"ok":false,"error":"code"}.
 * Input must be valid for len bytes; the caller owns and clears its credentials.
 */
void *t3_client_new(void);
char *t3_client_call(void *handle, const uint8_t *input, size_t len);
/* Upload read/release callbacks may run on an HTTP worker. With both callbacks
 * present, ownership of context transfers even on invalid input. release runs
 * exactly once when Rust drops the reader. Return 0 for EOF or -1 for error;
 * never return more than capacity. Callbacks must not unwind or throw. */
typedef ptrdiff_t (*t3_read_callback)(void *context, uint8_t *buffer, size_t capacity);
typedef void (*t3_release_callback)(void *context);
char *t3_client_upload(void *handle, const uint8_t *metadata, size_t len,
    t3_read_callback read, t3_release_callback release, void *context);
/* Metadata: {id,name,mimeType}. Validated image bytes are borrowed during the
 * callback, on the calling thread. Copy before returning; host owns context. */
typedef void (*t3_image_callback)(void *context, const uint8_t *bytes, size_t len);
char *t3_client_image(void *handle, const uint8_t *metadata, size_t len,
    t3_image_callback receive, void *context);
void t3_string_free(char *response);
void t3_client_free(void *handle);
#ifdef __cplusplus
}
#endif
#endif
