package dev.avillagran.t3

import java.io.Closeable
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.locks.ReentrantReadWriteLock
import kotlin.concurrent.read
import kotlin.concurrent.write
import org.json.JSONArray
import org.json.JSONObject

/** Blocking native transport. Use a worker/Dispatchers.IO; the host owns UI and lifecycle. */
class T3Client : Closeable {
    private val lifetime = ReentrantReadWriteLock()
    private val closing = AtomicBoolean(false)
    private var handle: Long = T3Native.create().also { check(it != 0L) { "native_error" } }

    private fun <T> useHandle(block: (Long) -> T): T = lifetime.read {
        check(!closing.get() && handle != 0L) { "client_closed" }
        block(handle)
    }
    private fun decode(bytes: ByteArray): Any? {
        val result = JSONObject(bytes.toString(Charsets.UTF_8))
        if (!result.getBoolean("ok")) throw T3Exception(result.optString("error", "native_error"))
        return result.opt("data").takeUnless { it === JSONObject.NULL }
    }
    fun execute(request: JSONObject): Any? = useHandle { native ->
        val input = request.toString().toByteArray(Charsets.UTF_8)
        try { decode(T3Native.call(native, input)) } finally { input.fill(0) }
    }
    fun connectBearer(baseUrl: String, token: String) {
        execute(JSONObject().put("op", "connectBearer").put("httpBaseUrl", baseUrl).put("token", token))
    }
    fun connectRelay(relayUrl: String, cloudJwt: String, environmentId: String) {
        execute(JSONObject().put("op", "connectRelay").put("relayUrl", relayUrl)
            .put("cloudJwt", cloudJwt).put("environmentId", environmentId).put("clientId", "t3-mobile"))
    }
    fun request(path: String, protocol: Boolean = true): Any? = execute(
        JSONObject().put("op", "request").put("method", "GET").put("path", path).put("protocol", protocol)
    )
    fun rpc(tag: String, payload: JSONObject): Any? = execute(
        JSONObject().put("op", "rpc").put("tag", tag).put("payload", payload)
    )
    fun subscribe(tag: String, payload: JSONObject): T3Subscription {
        val data = execute(JSONObject().put("op", "subscribe").put("tag", tag).put("payload", payload)) as JSONObject
        return T3Subscription(this, data.getString("subscriptionId"))
    }
    /** fd is borrowed for this call; JNI duplicates it and Rust streams without a full copy. */
    fun upload(fd: Int, name: String, mimeType: String, sizeBytes: Long): JSONObject = useHandle { native ->
        val metadata = JSONObject().put("name", name).put("mimeType", mimeType).put("sizeBytes", sizeBytes)
        decode(T3Native.upload(native, metadata.toString().toByteArray(Charsets.UTF_8), fd)) as JSONObject
    }
    fun image(id: String, name: String, mimeType: String): ByteArray = useHandle { native ->
        val metadata = JSONObject().put("id", id).put("name", name).put("mimeType", mimeType)
        val result = T3Native.image(native, metadata.toString().toByteArray(Charsets.UTF_8))
        decode(requireNotNull(result[0]))
        requireNotNull(result[1]) { "missing_image_bytes" }
    }
    override fun close() {
        if (!closing.compareAndSet(false, true)) return
        // Wake long-polls before waiting for the last in-flight native caller.
        try {
            lifetime.read {
                if (handle != 0L) T3Native.call(handle, "{\"op\":\"disconnect\"}".toByteArray(Charsets.UTF_8))
            }
        } finally { lifetime.write { if (handle != 0L) { T3Native.free(handle); handle = 0L } } }
    }
}
class T3Exception(val code: String) : Exception(code)
data class T3Batch(val items: JSONArray, val closed: Boolean, val error: String?)
class T3Subscription internal constructor(private val client: T3Client, private val id: String) : Closeable {
    private val closed = AtomicBoolean(false)
    /** One consumer only. Waits until data arrives, closes, or timeout (0..1000 ms). */
    fun poll(timeoutMs: Int = 1000): T3Batch {
        check(!closed.get()) { "subscription_closed" }
        require(timeoutMs in 0..1000)
        val value = client.execute(JSONObject().put("op", "pollSubscription").put("subscriptionId", id)
            .put("timeoutMs", timeoutMs)) as JSONObject
        return T3Batch(value.getJSONArray("items"), value.getBoolean("closed"),
            value.optString("error").takeIf { !value.isNull("error") && it.isNotEmpty() })
    }
    override fun close() {
        if (closed.compareAndSet(false, true)) runCatching {
            client.execute(JSONObject().put("op", "cancelSubscription").put("subscriptionId", id))
        }
    }
}
internal object T3Native {
    init { System.loadLibrary("t3_jni") }
    @JvmStatic external fun create(): Long
    @JvmStatic external fun free(handle: Long)
    @JvmStatic external fun call(handle: Long, request: ByteArray): ByteArray
    @JvmStatic external fun upload(handle: Long, metadata: ByteArray, fd: Int): ByteArray
    @JvmStatic external fun image(handle: Long, metadata: ByteArray): Array<ByteArray?>
}
