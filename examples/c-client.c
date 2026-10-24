/* Offline ABI smoke test. No credentials, network or user data required. */
#include "t3_protocol.h"
#include <assert.h>
#include <string.h>
static int released = 0;
static ptrdiff_t never_read(void *context, uint8_t *buffer, size_t capacity) {
    (void)context; (void)buffer; (void)capacity; return -1;
}
static void release_reader(void *context) { (void)context; released++; }
static void no_image(void *context, const uint8_t *bytes, size_t length) {
    (void)context; (void)bytes; (void)length;
}
int main(void) {
    void *client = t3_client_new();
    assert(client);
    const char *request = "{\"op\":\"request\",\"method\":\"GET\",\"path\":\"/api/orchestration/shell\"}";
    char *response = t3_client_call(client, (const uint8_t *)request, strlen(request));
    assert(response && strstr(response, "\"ok\":false") && strstr(response, "not_connected"));
    t3_string_free(response);
    const char *slow = "{\"op\":\"rpcWithTimeout\",\"tag\":\"device.list\",\"payload\":{},\"timeoutMs\":120000}";
    response = t3_client_call(client, (const uint8_t *)slow, strlen(slow));
    assert(response && strstr(response, "not_connected"));
    t3_string_free(response);
    const char *unbounded = "{\"op\":\"rpcWithTimeout\",\"tag\":\"device.list\",\"payload\":{},\"timeoutMs\":300001}";
    response = t3_client_call(client, (const uint8_t *)unbounded, strlen(unbounded));
    assert(response && strstr(response, "\"error\":\"invalid_rpc_timeout\""));
    t3_string_free(response);
    const char *capabilities = "{\"op\":\"capabilities\"}";
    response = t3_client_call(client, (const uint8_t *)capabilities, strlen(capabilities));
    assert(response && strstr(response, "\"rpcWithTimeout\""));
    t3_string_free(response);
    response = t3_client_call(client, (const uint8_t *)capabilities, strlen(capabilities));
    assert(response && strstr(response, "\"abiVersion\":3") && strstr(response, "\"assetBytes\"")
        && strstr(response, "\"pairBearer\"") && strstr(response, "\"fetchDescriptor\"")
        && strstr(response, "\"boundedThread\""));
    t3_string_free(response);
    response = t3_client_call(client, (const uint8_t *)capabilities, strlen(capabilities));
    assert(response && strstr(response, "\"wakeSubscriptions\"") && strstr(response, "\"maxSubscriptions\":8")
        && strstr(response, "\"maxLargeSubscriptions\":2") && strstr(response, "\"largeItemMaxBytes\":67108864")
        && strstr(response, "\"boundaries\"") && strstr(response, "\"completionMarker\""));
    t3_string_free(response);
    const char *wake = "{\"op\":\"wakeSubscriptions\"}";
    response = t3_client_call(client, (const uint8_t *)wake, strlen(wake));
    assert(response && strstr(response, "\"probed\":0"));
    t3_string_free(response);
    const char *bad_options = "{\"op\":\"connectBearer\",\"httpBaseUrl\":\"http://127.0.0.1:9\",\"token\":\"fixture-secret\",\"options\":{\"requestTimeoutMs\":0}}";
    response = t3_client_call(client, (const uint8_t *)bad_options, strlen(bad_options));
    assert(response && strstr(response, "\"error\":\"invalid_connection_options\""));
    t3_string_free(response);
    const char *bounded = "{\"op\":\"boundedThread\",\"threadId\":\"t\"}";
    response = t3_client_call(client, (const uint8_t *)bounded, strlen(bounded));
    assert(response && strstr(response, "not_connected"));
    t3_string_free(response);
    const char *descriptor = "{\"op\":\"fetchDescriptor\",\"httpBaseUrl\":\"https://host/prefix\"}";
    response = t3_client_call(client, (const uint8_t *)descriptor, strlen(descriptor));
    assert(response && strstr(response, "\"error\":\"invalid_endpoint\""));
    t3_string_free(response);
    const char *pair = "{\"op\":\"pairBearer\",\"httpBaseUrl\":\"http://127.0.0.1:9\",\"pairingCode\":\" \"}";
    response = t3_client_call(client, (const uint8_t *)pair, strlen(pair));
    assert(response && strstr(response, "\"error\":\"invalid_pairing_code\""));
    t3_string_free(response);
    const char *asset = "{\"op\":\"assetBytes\",\"resource\":{\"_tag\":\"attachment\"},\"maxBytes\":1024}";
    response = t3_client_call(client, (const uint8_t *)asset, strlen(asset));
    assert(response && strstr(response, "\"error\":\"binary_operation\""));
    t3_string_free(response);
    response = t3_client_image(client, (const uint8_t *)asset, strlen(asset), no_image, NULL);
    assert(response && strstr(response, "not_connected"));
    t3_string_free(response);
    response = t3_client_call(client, (const uint8_t *)"{", 1);
    assert(response && strstr(response, "malformed"));
    t3_string_free(response);
    const char *metadata = "{\"name\":\"fixture.txt\",\"mimeType\":\"text/plain\",\"sizeBytes\":7}";
    response = t3_client_upload(client, (const uint8_t *)metadata, strlen(metadata), never_read, release_reader, NULL);
    assert(response && strstr(response, "not_connected") && released == 1);
    t3_string_free(response);
    response = t3_client_image(client, (const uint8_t *)"{}", 2, no_image, NULL);
    assert(response && strstr(response, "not_connected"));
    t3_string_free(response);
    /* ABI 3: describe works offline; call validates before needing a connection. */
    const char *describe = "{\"op\":\"describe\"}";
    response = t3_client_call(client, (const uint8_t *)describe, strlen(describe));
    assert(response && strstr(response, "\"ok\":true") && strstr(response, "\"fn\":\"vcs.refreshStatus\""));
    t3_string_free(response);
    response = t3_client_call(client, (const uint8_t *)capabilities, strlen(capabilities));
    assert(response && strstr(response, "\"abiVersion\":3") && strstr(response, "\"call\"") && strstr(response, "\"describe\""));
    t3_string_free(response);
    const char *unknown = "{\"op\":\"call\",\"fn\":\"vcs.nope\",\"args\":{}}";
    response = t3_client_call(client, (const uint8_t *)unknown, strlen(unknown));
    assert(response && strstr(response, "\"error\":\"unknown_function\""));
    t3_string_free(response);
    const char *not_object = "{\"op\":\"call\",\"fn\":\"vcs.refreshStatus\",\"args\":[]}";
    response = t3_client_call(client, (const uint8_t *)not_object, strlen(not_object));
    assert(response && strstr(response, "\"error\":\"invalid_input\""));
    t3_string_free(response);
    const char *empty_cwd = "{\"op\":\"call\",\"fn\":\"vcs.refreshStatus\",\"args\":{\"cwd\":\"\"}}";
    response = t3_client_call(client, (const uint8_t *)empty_cwd, strlen(empty_cwd));
    assert(response && strstr(response, "\"error\":\"invalid_cwd\""));
    t3_string_free(response);
    /* Streams by function name: validated offline too; a stream is never a call. */
    response = t3_client_call(client, (const uint8_t *)describe, strlen(describe));
    assert(response && strstr(response, "\"streams\"") && strstr(response, "\"cursor\":\"afterSequence\""));
    t3_string_free(response);
    const char *bad_thread = "{\"op\":\"subscribe\",\"fn\":\"orchestration.subscribeThread\",\"args\":{\"threadId\":\"\"}}";
    response = t3_client_call(client, (const uint8_t *)bad_thread, strlen(bad_thread));
    assert(response && strstr(response, "\"error\":\"invalid_thread_id\""));
    t3_string_free(response);
    const char *unary = "{\"op\":\"subscribe\",\"fn\":\"vcs.refreshStatus\",\"args\":{\"cwd\":\"/r\"}}";
    response = t3_client_call(client, (const uint8_t *)unary, strlen(unary));
    assert(response && strstr(response, "\"error\":\"not_a_stream\""));
    t3_string_free(response);
    const char *stream_call = "{\"op\":\"call\",\"fn\":\"orchestration.subscribeShell\",\"args\":{}}";
    response = t3_client_call(client, (const uint8_t *)stream_call, strlen(stream_call));
    assert(response && strstr(response, "\"error\":\"stream_function\""));
    t3_string_free(response);
    const char *shell = "{\"op\":\"subscribe\",\"fn\":\"orchestration.subscribeShell\",\"args\":{\"afterSequence\":3}}";
    response = t3_client_call(client, (const uint8_t *)shell, strlen(shell));
    assert(response && strstr(response, "\"error\":\"not_connected\""));
    t3_string_free(response);
    const char *valid = "{\"op\":\"call\",\"fn\":\"vcs.refreshStatus\",\"args\":{\"cwd\":\"/r\"}}";
    response = t3_client_call(client, (const uint8_t *)valid, strlen(valid));
    assert(response && strstr(response, "\"error\":\"not_connected\""));
    t3_string_free(response);
    t3_client_free(client);
    t3_client_free(NULL);
    t3_string_free(NULL);
    return 0;
}
