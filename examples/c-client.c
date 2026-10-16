/* Offline ABI smoke test. No credentials, network or user data required. */
#include "t3_protocol.h"
#include <assert.h>
#include <string.h>
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
    t3_client_free(client);
    t3_client_free(NULL);
    t3_string_free(NULL);
    return 0;
}
