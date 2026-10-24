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
        if (!result.getBoolean("ok")) throw T3Exception(
            result.optString("error", "native_error"), result.text("message"), result.text("requiredScope")
        )
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
    /** Typed API (ABI 3): validated in Rust before any socket, sent once, never retried.
     *  `fn` and `args` follow [describe]; `timeoutMs` (1..300000) overrides the function's deadline. */
    fun call(fn: String, args: JSONObject = JSONObject(), timeoutMs: Int? = null): Any? = execute(
        JSONObject().put("op", "call").put("fn", fn).put("args", args).put("timeoutMs", timeoutMs)
    )
    /** The typed API registry: functions, args, kind, replay rule, scope and deadline. Offline. */
    fun describe(): JSONObject = execute(JSONObject().put("op", "describe")) as JSONObject
    /** Up to 8 per handle (2 of them bounded thread streams); more fail with `subscription_limit`. */
    fun subscribe(tag: String, payload: JSONObject): T3Subscription {
        val data = execute(JSONObject().put("op", "subscribe").put("tag", tag).put("payload", payload)) as JSONObject
        return T3Subscription(this, data.getString("subscriptionId"))
    }
    /**
     * Probes every open stream of this handle, e.g. on resume or a ConnectivityManager network
     * change: each pings at once and ends with `stream_timeout` unless the server answers within
     * 3 s. Returns how many streams were probed. Never reconnects; that stays with the host.
     */
    fun wakeSubscriptions(): Int =
        (execute(JSONObject().put("op", "wakeSubscriptions")) as JSONObject).getInt("probed")
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
/** `code` is stable; `reason` is the server's one-line message; `requiredScope`
 *  names the scope a `write_not_authorized` token lacked (the host picks tokens). */
class T3Exception(val code: String, val reason: String? = null, val requiredScope: String? = null) :
    Exception(if (reason == null) code else "$code: $reason")
private fun JSONObject.text(key: String): String? = optString(key).takeIf { !isNull(key) && it.isNotEmpty() }

/**
 * One poll of a subscription.
 * - [items]: native T3 items in server order, never dropped or reordered.
 * - [boundaries]: strictly ascending end offsets into [items]. Each `n` means a server chunk
 *   ended after `items[0 until n]`; apply and repaint there. Items after the last boundary
 *   belong to a chunk that continues in the next batch; `0` means the chunk that ended the
 *   previous batch ended right there. [T3Chunks] does this regrouping.
 * - [idle]: the last chunk is complete and no new frame completed during a 100 ms read tick:
 *   caught up for now (a large frame may still be in transit; it can be true before the first
 *   item). The only catch-up signal when `requestCompletionMarker` was not requested.
 * - [completionMarker]: with `requestCompletionMarker`, true from the batch that carries the
 *   server's `{"kind":"synchronized"}` item (still in [items]) onward: paint the initial state.
 * - [closed]: nothing more will arrive. [error] is a stable code (null after a cancel) and
 *   [errorMessage] an optional reason. On `stream_timeout`/`disconnected` close this
 *   subscription and subscribe again from the host's cursor, with the host's backoff.
 */
data class T3Batch(
    val items: JSONArray,
    val closed: Boolean,
    val error: String?,
    val errorMessage: String? = null,
    val boundaries: List<Int> = emptyList(),
    val idle: Boolean = false,
    val completionMarker: Boolean = false,
)

/** Regroups polled items into the server's chunks, carrying a partial chunk across polls. */
class T3Chunks {
    private val open = mutableListOf<Any?>()
    /** The chunks [batch] completes, each a list of items in server order. */
    fun accept(batch: T3Batch): List<List<Any?>> {
        val done = mutableListOf<List<Any?>>()
        var start = 0
        for (end in batch.boundaries) {
            for (i in start until end) open.add(batch.items.opt(i))
            done.add(open.toList())
            open.clear()
            start = end
        }
        for (i in start until batch.items.length()) open.add(batch.items.opt(i))
        return done
    }
    /** Items received after the last chunk end. */
    val pending: List<Any?> get() = open.toList()
}
class T3Subscription internal constructor(private val client: T3Client, private val id: String) : Closeable {
    private val closed = AtomicBoolean(false)
    /**
     * One consumer only. Returns as soon as something is new (items, a chunk end, idle turning
     * true, or the end of the stream), otherwise after [timeoutMs] (0..1000 ms).
     */
    fun poll(timeoutMs: Int = 1000): T3Batch {
        check(!closed.get()) { "subscription_closed" }
        require(timeoutMs in 0..1000)
        val value = client.execute(JSONObject().put("op", "pollSubscription").put("subscriptionId", id)
            .put("timeoutMs", timeoutMs)) as JSONObject
        val ends = value.optJSONArray("boundaries")
        return T3Batch(
            items = value.getJSONArray("items"),
            closed = value.getBoolean("closed"),
            error = value.text("error"),
            errorMessage = value.text("errorMessage"),
            boundaries = if (ends == null) emptyList() else List(ends.length()) { ends.getInt(it) },
            idle = value.optBoolean("idle"),
            completionMarker = value.optBoolean("completionMarker"),
        )
    }
    /**
     * Probes only this stream (see [T3Client.wakeSubscriptions]). False when it already ended;
     * a dead socket then shows up as a closed batch with `stream_timeout` within about 3 s.
     */
    fun wake(): Boolean {
        check(!closed.get()) { "subscription_closed" }
        val value = client.execute(JSONObject().put("op", "wakeSubscriptions").put("subscriptionId", id)) as JSONObject
        return value.getInt("probed") == 1
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
