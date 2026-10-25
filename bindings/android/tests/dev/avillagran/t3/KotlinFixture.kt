package dev.avillagran.t3

import com.sun.net.httpserver.HttpServer
import java.io.File
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
        val saved = JSONObject(File("bindings/android/describe.json").readText()).getJSONArray("functions").length()
        val described = client.describe().getJSONArray("functions").length()
        check(described == saved && described > 100) { "describe lists $described functions, describe.json $saved" }
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
        // Domain functions are pure: they answer offline, with the same error envelope.
        val domains = client.describe().getJSONArray("domain").length()
        val savedDomains = JSONObject(File("bindings/android/describe.json").readText()).getJSONArray("domain").length()
        check(domains == savedDomains && domains > 20) { "describe lists $domains domain functions, describe.json $savedDomains" }
        val blocks = api.domain.codeBlocks.parse("```bash\necho hi\n```") as org.json.JSONArray
        check(blocks.getJSONObject(0).getString("command") == "echo hi") { blocks.toString() }
        val target = api.domain.connectionCatalog.pairingTarget("wss://node:3773/x?token=abc") as JSONObject
        check(target.getString("httpBaseUrl") == "https://node:3773" && target.getString("pairingCode") == "abc")
        val queue = api.domain.queue.derive(JSONObject().put("runs", org.json.JSONArray())) as JSONObject
        check(queue.getJSONArray("items").length() == 0 && !queue.getBoolean("canSteer"))
        val shell = JSONObject()
            .put("projects", org.json.JSONArray().put(JSONObject().put("id", "p").put("title", "omarchy-t3-code")))
            .put("threads", org.json.JSONArray().put(JSONObject().put("id", "t-1").put("title", "Fix build")
                .put("projectId", "p").put("status", "running").put("latestRunId", "r").put("updatedAt", "2026-10-15T04:00:00Z")))
        val environments = org.json.JSONArray()
            .put(JSONObject().put("id", "laptop").put("name", "Laptop").put("shell", shell))
            .put(JSONObject().put("id", "desktop").put("state", "unreachable"))
        val sidebar = api.domain.sidebar.view(environments, 1792036800000L, grouped = true, colors = JSONObject().put("blue", "#123456")) as JSONObject
        val working = sidebar.getJSONObject("sections").getJSONArray("working").getJSONObject(0)
        check(working.getString("key") == "laptop/t-1" && working.getString("detail") == "working" && working.getString("statusLabel") == "Working")
        check(sidebar.getBoolean("stale") && sidebar.getJSONObject("projectIdentities").getJSONObject("laptop/p").getString("monogram") == "OC")
        val projection = JSONObject().put("thread", JSONObject().put("id", "t-1"))
            .put("messages", org.json.JSONArray().put(JSONObject().put("id", "m").put("role", "assistant").put("text", "**Done** in `src/lib.rs`").put("createdAt", 1)))
        val applied = api.domain.projection.apply(projection, events = org.json.JSONArray()
            .put(JSONObject().put("type", "run.updated").put("payload", JSONObject().put("id", "r").put("status", "running")))) as JSONObject
        check(applied.getBoolean("changed"))
        val thread = api.domain.timeline.view(applied.getJSONObject("projection")) as JSONObject
        val reply = thread.getJSONArray("timeline").getJSONObject(0)
        check(reply.getString("rev").length == 16 && thread.getJSONObject("threadState").getBoolean("working"))
        val spans = reply.getJSONArray("blocks").getJSONObject(0).getJSONArray("spans")
        check(spans.getJSONObject(0).getBoolean("strong") && spans.getJSONObject(2).getJSONObject("link").getString("fileType") == "rust") { spans.toString() }
        val md = api.domain.markdown.blocks("[x](javascript:alert(1)) [gh](https://github.com/a/b)") as org.json.JSONArray
        check(!md.getJSONObject(0).getJSONArray("spans").getJSONObject(0).has("link"))
        check((api.domain.markdown.link("https://github.com/a/b") as JSONObject).getString("kind") == "github")
        val chips = api.domain.projects.identity(org.json.JSONArray().put(JSONObject().put("key", "e/p").put("title", "lib t3 rust"))) as org.json.JSONArray
        check(chips.getJSONObject(0).getString("monogram") == "LR")
        val noDomain = runCatching { client.domain("queue.nope") }.exceptionOrNull()
        check(noDomain is T3Exception && noDomain.code == "unknown_function")
        val badDomain = runCatching { client.domain("sidebar.command", JSONObject().put("action", "nap").put("threadId", "t")) }.exceptionOrNull()
        check(badDomain is T3Exception && badDomain.code == "invalid_action") { badDomain.toString() }
        // JNI's own failures arrive as T3Exception too, before Rust or a connection.
        val badFd = runCatching { client.upload(-1, "x.bin", "application/octet-stream", 1) }.exceptionOrNull()
        check(badFd is T3Exception && badFd.code == "host_file_unavailable") { badFd.toString() }
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
        println("KOTLIN_JNI_PASS: canonical wrapper, capabilities, typed API call/describe, domain functions, asset wrappers, connection options, Unicode, stable and JNI errors and concurrent close ownership")
    } finally { finish.countDown(); client.close(); server.stop(0) }
}
