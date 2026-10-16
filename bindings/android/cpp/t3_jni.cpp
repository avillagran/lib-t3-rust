// Marshaling only. HTTP, WebSockets, authentication and attachment grants stay Rust.
#include <jni.h>
#include "t3_protocol.h"
#include <cerrno>
#include <chrono>
#include <fcntl.h>
#include <poll.h>
#include <cstring>
#include <memory>
#include <new>
#include <unistd.h>
#include <vector>
namespace {
struct Input : std::vector<uint8_t> {
    ~Input() { volatile uint8_t *bytes = data(); for (size_t n = 0; n < size(); ++n) bytes[n] = 0; }
};
using Response = std::unique_ptr<char, decltype(&t3_string_free)>;
void fail(JNIEnv *env, const char *code) {
    jclass type = env->FindClass("java/lang/IllegalStateException");
    if (type) env->ThrowNew(type, code);
}
bool input(JNIEnv *env, jbyteArray bytes, std::vector<uint8_t> &out) {
    if (!bytes || env->GetArrayLength(bytes) > 8 * 1024 * 1024) { fail(env, "invalid_input"); return false; }
    out.resize(static_cast<size_t>(env->GetArrayLength(bytes)));
    env->GetByteArrayRegion(bytes, 0, static_cast<jsize>(out.size()), reinterpret_cast<jbyte *>(out.data()));
    return !env->ExceptionCheck();
}
jbyteArray bytes(JNIEnv *env, const uint8_t *data, size_t size) {
    auto result = env->NewByteArray(static_cast<jsize>(size));
    if (result && size) env->SetByteArrayRegion(result, 0, static_cast<jsize>(size), reinterpret_cast<const jbyte *>(data));
    return result;
}
jbyteArray response(JNIEnv *env, char *raw) {
    Response owned(raw, t3_string_free);
    if (!raw) { fail(env, "native_error"); return nullptr; }
    return bytes(env, reinterpret_cast<const uint8_t *>(raw), std::strlen(raw));
}
struct FileInput { int fd; std::chrono::steady_clock::time_point deadline; };
ptrdiff_t read_file(void *opaque, uint8_t *buffer, size_t capacity) {
    auto *file = static_cast<FileInput *>(opaque);
    while (true) {
        if (std::chrono::steady_clock::now() >= file->deadline) return -1;
        pollfd waiting{file->fd, POLLIN, 0};
        int ready = ::poll(&waiting, 1, 1000);
        if (ready > 0) break;
        if (ready < 0 && errno != EINTR) return -1;
    }
    ssize_t n;
    do { n = ::read(static_cast<FileInput *>(opaque)->fd, buffer, capacity); } while (n < 0 && errno == EINTR);
    return static_cast<ptrdiff_t>(n);
}
void release_file(void *opaque) {
    auto *file = static_cast<FileInput *>(opaque);
    ::close(file->fd);
    delete file;
}
struct ImageOutput { JNIEnv *env; jbyteArray value; };
void image(void *opaque, const uint8_t *data, size_t size) {
    auto *out = static_cast<ImageOutput *>(opaque);
    out->value = bytes(out->env, data, size);
}
}
extern "C" JNIEXPORT jlong JNICALL Java_dev_avillagran_t3_T3Native_create(JNIEnv *, jclass) {
    return reinterpret_cast<jlong>(t3_client_new());
}
extern "C" JNIEXPORT void JNICALL Java_dev_avillagran_t3_T3Native_free(JNIEnv *, jclass, jlong handle) {
    t3_client_free(reinterpret_cast<void *>(handle));
}
extern "C" JNIEXPORT jbyteArray JNICALL Java_dev_avillagran_t3_T3Native_call(JNIEnv *env, jclass, jlong handle, jbyteArray data) {
    try {
        Input request;
        if (!input(env, data, request)) return nullptr;
        return response(env, t3_client_call(reinterpret_cast<void *>(handle), request.data(), request.size()));
    } catch (const std::bad_alloc &) { fail(env, "native_allocation_failed"); return nullptr; }
}
extern "C" JNIEXPORT jbyteArray JNICALL Java_dev_avillagran_t3_T3Native_upload(JNIEnv *env, jclass, jlong handle, jbyteArray data, jint fd) {
    try {
        Input request;
        if (!input(env, data, request)) return nullptr;
        // Own a duplicate until Rust releases its reader, including every error path.
        const int duplicate = ::fcntl(fd, F_DUPFD_CLOEXEC, 0);
        if (duplicate < 0) { fail(env, "host_file_unavailable"); return nullptr; }
        auto *file = new (std::nothrow) FileInput{duplicate, std::chrono::steady_clock::now() + std::chrono::seconds(90)};
        if (!file) { ::close(duplicate); fail(env, "native_allocation_failed"); return nullptr; }
        return response(env, t3_client_upload(reinterpret_cast<void *>(handle), request.data(), request.size(), read_file, release_file, file));
    } catch (const std::bad_alloc &) { fail(env, "native_allocation_failed"); return nullptr; }
}
extern "C" JNIEXPORT jobjectArray JNICALL Java_dev_avillagran_t3_T3Native_image(JNIEnv *env, jclass, jlong handle, jbyteArray data) {
    try {
        Input request;
        if (!input(env, data, request)) return nullptr;
        ImageOutput out{env, nullptr};
        Response raw(t3_client_image(reinterpret_cast<void *>(handle), request.data(), request.size(), image, &out), t3_string_free);
        if (env->ExceptionCheck()) return nullptr;
        auto status = bytes(env, reinterpret_cast<const uint8_t *>(raw.get()), std::strlen(raw.get()));
        if (env->ExceptionCheck()) return nullptr;
        auto type = env->FindClass("[B");
        if (!type) return nullptr;
        auto result = env->NewObjectArray(2, type, nullptr);
        if (!result) return nullptr;
        env->SetObjectArrayElement(result, 0, status);
        env->SetObjectArrayElement(result, 1, out.value);
        return result;
    } catch (const std::bad_alloc &) { fail(env, "native_allocation_failed"); return nullptr; }
}
