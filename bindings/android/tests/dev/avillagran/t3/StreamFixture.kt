package dev.avillagran.t3

import java.io.ByteArrayOutputStream
import java.io.DataInputStream
import java.io.IOException
import java.net.InetAddress
import java.net.ServerSocket
import java.net.Socket
import java.security.MessageDigest
import java.util.Base64
import java.util.concurrent.atomic.AtomicReference
import org.json.JSONArray
import org.json.JSONObject

/**
 * Server side of one fixture WebSocket (RFC 6455, text frames, no extensions). Enough to
 * script T3's Effect RPC stream frames; never a real T3 server.
 */
private class FakeSocket(private val socket: Socket) {
    private val input = DataInputStream(socket.getInputStream().buffered())
    private val output = socket.getOutputStream()

    init {
        socket.soTimeout = 10_000
        val head = StringBuilder()
        while (!head.endsWith("\r\n\r\n")) head.append(input.readUnsignedByte().toChar())
        check(head.startsWith("GET /ws")) { head.toString() }
        val key = head.lines().first { it.startsWith("Sec-WebSocket-Key:", ignoreCase = true) }
            .substringAfter(':').trim()
        val accept = Base64.getEncoder().encodeToString(MessageDigest.getInstance("SHA-1")
            .digest((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").toByteArray(Charsets.US_ASCII)))
        output.write(("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n" +
            "Sec-WebSocket-Accept: $accept\r\n\r\n").toByteArray(Charsets.US_ASCII))
        output.flush()
    }

    /** Next client text frame as JSON; null once the client closes or the socket ends. */
    fun read(): JSONObject? {
        while (true) {
            val first = try { input.readUnsignedByte() } catch (end: IOException) { return null }
            val second = input.readUnsignedByte()
            var length = (second and 0x7f).toLong()
            if (length == 126L) length = input.readUnsignedShort().toLong()
            else if (length == 127L) length = input.readLong()
            val masked = (second and 0x80) != 0
            val mask = ByteArray(4)
            if (masked) input.readFully(mask)
            val payload = ByteArray(length.toInt())
            input.readFully(payload)
            if (masked) for (i in payload.indices) payload[i] = (payload[i].toInt() xor mask[i % 4].toInt()).toByte()
            when (first and 0x0f) {
                0x1 -> return JSONObject(payload.toString(Charsets.UTF_8))
                0x8 -> return null
                else -> {} // control frames the fixture does not need
            }
        }
    }

    fun send(frame: JSONObject) {
        val bytes = frame.toString().toByteArray(Charsets.UTF_8)
        val header = ByteArrayOutputStream()
        header.write(0x81)
        when {
            bytes.size < 126 -> header.write(bytes.size)
            bytes.size < 65536 -> { header.write(126); header.write(bytes.size shr 8); header.write(bytes.size and 0xff) }
            else -> { header.write(127); for (shift in 56 downTo 0 step 8) header.write(((bytes.size.toLong() shr shift) and 0xff).toInt()) }
        }
        output.write(header.toByteArray())
        output.write(bytes)
        output.flush()
    }

    fun chunk(vararg values: JSONObject) =
        send(JSONObject().put("_tag", "Chunk").put("requestId", "1").put("values", JSONArray(values.toList())))

    /** Reads until Interrupt or the end of the socket; answers Pings only when [answer]. */
    fun drain(answer: Boolean = false) {
        while (true) {
            val frame = read() ?: return
            when (frame.optString("_tag")) {
                "Interrupt" -> return
                "Ping" -> if (answer) send(JSONObject().put("_tag", "Pong"))
            }
        }
    }

    fun close() = socket.close()
}

private val failure = AtomicReference<Throwable?>(null)

/** One fake subscription socket: [script] gets the client's Request and the socket. */
private fun serve(script: (JSONObject, FakeSocket) -> Unit): Pair<String, Thread> {
    val server = ServerSocket(0, 1, InetAddress.getLoopbackAddress())
    val thread = Thread {
        try {
            server.use {
                val ws = FakeSocket(it.accept())
                try {
                    val request = checkNotNull(ws.read()) { "no Request frame" }
                    check(request.getString("_tag") == "Request") { request.toString() }
                    script(request, ws)
                } finally { ws.close() }
            }
        } catch (error: Throwable) { failure.compareAndSet(null, error) }
    }
    thread.isDaemon = true
    thread.start()
    return "http://127.0.0.1:${server.localPort}" to thread
}

private fun JSONArray.strings() = List(length()) { getString(it) }

private fun item(vararg pairs: Pair<String, Any>) = JSONObject().apply { for ((k, v) in pairs) put(k, v) }

/** Polls until [done] holds for a batch or the stream closes; returns every batch. */
private fun T3Subscription.pollUntil(done: (T3Batch) -> Boolean): List<T3Batch> {
    val batches = mutableListOf<T3Batch>()
    val deadline = System.nanoTime() + 10_000_000_000L
    while (System.nanoTime() < deadline) {
        val batch = poll()
        batches.add(batch)
        if (done(batch) || batch.closed) return batches
    }
    error("stream never finished: $batches")
}

private fun chunksAndMarker() {
    val (base, server) = serve { request, ws ->
        val payload = request.getJSONObject("payload")
        check(payload.getLong("afterSequence") == 41L && payload.getBoolean("requestCompletionMarker")) { payload.toString() }
        ws.chunk(item("kind" to "snapshot", "snapshot" to item("snapshotSequence" to 41)), item("kind" to "synchronized"))
        check(ws.read()?.getString("_tag") == "Ack")
        ws.chunk(item("kind" to "thread.updated", "sequence" to 42), item("kind" to "thread.removed", "sequence" to 43))
        check(ws.read()?.getString("_tag") == "Ack")
        ws.drain()
    }
    T3Client().use { client ->
        client.connectBearer(base, "fixture-token")
        val stream = client.subscribe("orchestration.subscribeShell",
            JSONObject().put("afterSequence", 41).put("requestCompletionMarker", true))
        val chunks = T3Chunks()
        val complete = mutableListOf<List<Any?>>()
        var received = 0
        var last: T3Batch
        do {
            last = stream.poll()
            check(!last.closed) { "${last.error}: ${last.errorMessage}" }
            received += last.items.length()
            complete.addAll(chunks.accept(last))
        } while (received < 4 || !last.idle)
        check(complete.map { it.size } == listOf(2, 2)) { complete.toString() }
        check(chunks.pending.isEmpty())
        check((complete[0][1] as JSONObject).getString("kind") == "synchronized")
        check(last.completionMarker && last.error == null && last.errorMessage == null)
        stream.close()
    }
    server.join(5000)
}

private fun wakeCutsASilentSocket() {
    val (base, server) = serve { _, ws -> ws.drain(answer = false) }
    T3Client().use { client ->
        client.connectBearer(base, "fixture-token")
        val stream = client.subscribe("orchestration.subscribeShell", JSONObject())
        stream.pollUntil { it.idle }
        val woke = System.nanoTime()
        check(stream.wake())
        val end = stream.pollUntil { false }.last()
        val elapsedMs = (System.nanoTime() - woke) / 1_000_000
        check(end.closed && end.error == "stream_timeout" && end.errorMessage == null) { end.toString() }
        check(elapsedMs in 2900..3400) { "cut after $elapsedMs ms" }
        check(!stream.wake()) // already ended
        check(client.wakeSubscriptions() == 0)
        stream.close()
    }
    server.join(5000)
}

private fun refusedStreamCarriesAReason() {
    val server = ServerSocket(0, 1, InetAddress.getLoopbackAddress())
    val thread = Thread {
        try {
            server.use {
                it.accept().use { socket ->
                    socket.soTimeout = 5000
                    val input = socket.getInputStream()
                    val head = StringBuilder()
                    while (!head.endsWith("\r\n\r\n")) head.append(input.read().also { b -> check(b >= 0) }.toChar())
                    socket.getOutputStream().write(
                        "HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".toByteArray(Charsets.US_ASCII))
                }
            }
        } catch (error: Throwable) { failure.compareAndSet(null, error) }
    }.apply { isDaemon = true; start() }
    T3Client().use { client ->
        client.connectBearer("http://127.0.0.1:${server.localPort}", "fixture-token")
        val end = client.subscribe("orchestration.subscribeShell", JSONObject()).pollUntil { false }.last()
        check(end.error == "rpc_connection_failed" && end.errorMessage.orEmpty().contains("401")) { end.toString() }
    }
    thread.join(5000)
}

private fun threadStreamResumesFromTheLibraryCursor() {
    val (base, server) = serve { request, ws ->
        check(request.getString("tag") == "orchestration.subscribeThread") { request.toString() }
        val payload = request.getJSONObject("payload")
        check(payload.getString("threadId") == "t-1" && payload.getLong("afterSequence") == 7L) { payload.toString() }
        check(payload.getBoolean("requestCompletionMarker") && payload.getBoolean("acceptBoundedSnapshot")) { payload.toString() }
        ws.chunk(item("kind" to "event", "sequence" to 8, "event" to item("type" to "message.updated")),
            item("kind" to "event", "sequence" to 9, "event" to item("type" to "future.event")))
        check(ws.read()?.getString("_tag") == "Ack")
        ws.chunk(item("kind" to "snapshot", "snapshotSequence" to 20, "projection" to item("id" to "t-1")),
            item("kind" to "synchronized"),
            item("kind" to "event", "sequence" to 21, "event" to item("type" to "turn.started")))
        check(ws.read()?.getString("_tag") == "Ack")
        ws.drain()
    }
    T3Client().use { client ->
        client.connectBearer(base, "fixture-token")
        // Typed args are checked before any socket opens.
        val rejected = runCatching { client.subscribeFn("orchestration.subscribeThread", JSONObject().put("threadId", "")) }
        check((rejected.exceptionOrNull() as? T3Exception)?.code == "invalid_thread_id") { rejected.toString() }
        ThreadStream.thread(client, "t-1", afterSequence = 7, completionMarker = true).use { stream ->
            check(stream.cursor == 7L)
            val updates = mutableListOf<ThreadUpdate>()
            val deadline = System.nanoTime() + 10_000_000_000L
            do {
                val update = stream.poll()
                check(!update.closed) { "${update.error}: ${update.errorMessage}" }
                updates.add(update)
                check(System.nanoTime() < deadline) { updates.toString() }
            } while (stream.cursor != 21L || !update.idle)
            check(stream.start == "resume") { stream.start.toString() }
            // Whatever the batching, the snapshot replaces what came before it.
            val ordered = updates.flatMap { u ->
                listOfNotNull(u.snapshot?.let { "S" + it.getLong("snapshotSequence") }) + u.events.map { it.getLong("sequence").toString() }
            }
            check(ordered.takeLast(2) == listOf("S20", "21") && ordered.dropLast(2) in listOf(emptyList(), listOf("8", "9"))) { ordered.toString() }
            check(updates.any { it.synchronized } && updates.last().completionMarker && updates.last().repaint)
        }
    }
    server.join(5000)
}

/** Streams through the canonical Kotlin wrapper and the real JNI library. */
object StreamFixture {
    @JvmStatic
    fun main(args: Array<String>) {
        T3Client().use { client ->
            val capabilities = client.execute(JSONObject().put("op", "capabilities")) as JSONObject
            check(capabilities.getInt("abiVersion") == 3)
            check(capabilities.getInt("maxSubscriptions") == 8 && capabilities.getInt("maxLargeSubscriptions") == 2)
            check("wakeSubscriptions" in capabilities.getJSONArray("operations").strings())
            check("boundaries" in capabilities.getJSONArray("pollFields").strings())
            check("cursor" in capabilities.getJSONArray("pollFields").strings())
            check("fn" in capabilities.getJSONArray("subscribeBy").strings())
        }
        chunksAndMarker()
        wakeCutsASilentSocket()
        refusedStreamCarriesAReason()
        threadStreamResumesFromTheLibraryCursor()
        failure.get()?.let { throw it }
        println("KOTLIN_STREAMS_PASS: chunk boundaries, completion marker, idle, wake probe, error reasons and ThreadStream cursor")
    }
}
