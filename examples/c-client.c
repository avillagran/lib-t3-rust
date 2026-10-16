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
    t3_client_free(client);
    t3_client_free(NULL);
    t3_string_free(NULL);
    return 0;
}
