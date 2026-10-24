package dev.avillagran.t3

import com.sun.net.httpserver.HttpServer
import java.net.InetSocketAddress
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicReference
import org.json.JSONObject

/** Executes the canonical Kotlin wrapper against the real JNI shared library. */
fun main() {
    val server = HttpServer.create(InetSocketAddress("127.0.0.1", 0), 0)
    val entered = CountDownLatch(1)
    val finish = CountDownLatch(1)
    server.createContext("/fixture") { exchange ->
        if (exchange.requestURI.path.endsWith("/slow")) {
            entered.countDown()
            check(finish.await(4, TimeUnit.SECONDS))
        }
        val bytes = "{\"text\":\"á 🦀\"}".toByteArray(Charsets.UTF_8)
        exchange.responseHeaders.set("Content-Type", "application/json")
        exchange.sendResponseHeaders(200, bytes.size.toLong())
        exchange.responseBody.use { it.write(bytes) }
    }
    server.start()
    val client = T3Client()
    try {
        check((client.execute(JSONObject().put("op", "capabilities")) as JSONObject).getInt("abiVersion") == 3)
        val disconnected = runCatching { client.request("/fixture") }.exceptionOrNull()
        check(disconnected is T3Exception && disconnected.code == "not_connected")
        // Typed API: describe is offline, validation runs before the connection
        // check, and one generated wrapper per module reaches Rust through JNI.
        check(client.describe().getJSONArray("functions").length() == 27)
        val api = T3Api(client)
        val invalid = runCatching { api.vcs.refreshStatus("") }.exceptionOrNull()
        check(invalid is T3Exception && invalid.code == "invalid_cwd")
        val unknown = runCatching { client.call("vcs.nope") }.exceptionOrNull()
        check(unknown is T3Exception && unknown.code == "unknown_function")
        val archive = JSONObject().put("type", "thread.archive").put("commandId", "fixture-1").put("threadId", "t-1")
        val attachment = JSONObject().put("_tag", "attachment").put("attachmentId", "a-1")
        for (call in listOf<() -> Any?>(
            { api.server.getConfig() }, { api.orchestration.dispatchCommand(archive) }, { api.preview.list("t-1") },
            { api.assets.createUrl(attachment) }, { api.vcs.refreshStatus("/repo") },
            { api.review.getDiffPreview("/repo", ignoreWhitespace = true) }, { api.projects.readFile("/repo", "README.md") },
        )) {
            val error = runCatching { call() }.exceptionOrNull()
            check(error is T3Exception && error.code == "not_connected") { error.toString() }
        }
        // Signed assets and favicons go through the binary native path, not call().
        for (call in listOf<() -> T3Asset>({ client.assetBytes(attachment) }, { client.projectFavicon("/repo") })) {
            val error = runCatching { call() }.exceptionOrNull()
            check(error is T3Exception && error.code == "not_connected") { error.toString() }
        }
        val base = "http://127.0.0.1:${server.address.port}"
        val badOptions = runCatching { client.connectBearer(base, "fixture-token", JSONObject().put("retries", 3)) }
        check((badOptions.exceptionOrNull() as? T3Exception)?.code == "invalid_connection_options") { badOptions.toString() }
        val relay = runCatching {
            client.connectRelay("https://relay.invalid", "jwt", "env-1", clientId = "fixture", options = JSONObject().put("requestTimeoutMs", 0))
        }
        check((relay.exceptionOrNull() as? T3Exception)?.code == "invalid_connection_options") { relay.toString() }
        client.connectBearer(base, "fixture-token", JSONObject().put("requestTimeoutMs", 5000).put("connectTimeoutMs", 2000))
        check((client.request("/fixture") as JSONObject).getString("text") == "á 🦀")
        val image = runCatching { client.image("fixture", "x.svg", "image/svg+xml") }.exceptionOrNull()
        check(image is T3Exception && image.code == "invalid_image")
        val timeout = runCatching { client.assetBytes(attachment, maxBytes = 10, timeoutMs = 0) }.exceptionOrNull()
        check(timeout is T3Exception && timeout.code == "invalid_asset_timeout") { timeout.toString() }
        val failure = AtomicReference<Throwable?>(null)
        val worker = Thread {
            try { check((client.request("/fixture/slow") as JSONObject).getString("text") == "á 🦀") }
            catch (error: Throwable) { failure.set(error) }
        }
        worker.start()
        check(entered.await(3, TimeUnit.SECONDS))
        val closing = Thread { try { client.close() } catch (error: Throwable) { failure.set(error) } }
        closing.start()
        Thread.sleep(50)
        check(closing.isAlive) { "native handle freed while a request was active" }
        finish.countDown()
        worker.join(3000); closing.join(3000)
        check(!worker.isAlive && !closing.isAlive && failure.get() == null) { failure.get().toString() }
        client.close() // idempotent
        check(runCatching { client.request("/fixture") }.exceptionOrNull() is IllegalStateException)
        println("KOTLIN_JNI_PASS: canonical wrapper, capabilities, typed API call/describe, asset wrappers, Unicode, stable errors and concurrent close ownership")
    } finally { finish.countDown(); client.close(); server.stop(0) }
}
