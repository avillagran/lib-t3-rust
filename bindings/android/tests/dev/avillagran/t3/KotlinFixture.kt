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
        val aware = api.domain.awareness.view(environments, 1792036800000L, locale = "es") as JSONObject
        val liveCard = aware.getJSONObject("card")
        check(liveCard.getBoolean("active") && liveCard.getJSONObject("chip").getString("kind") == "working" && aware.getJSONArray("alerts").length() == 0) { aware.toString() }
        val asking = JSONObject(shell.toString())
        asking.getJSONArray("threads").getJSONObject(0).put("pendingRuntimeRequest", JSONObject().put("id", "req-1").put("kind", "command_approval").put("createdAt", "2026-10-15T04:00:30Z"))
        val askEnvironments = org.json.JSONArray().put(JSONObject().put("id", "laptop").put("name", "Laptop").put("shell", asking))
        val alerted = api.domain.awareness.view(askEnvironments, 1792036860000L, baseline = aware.getJSONObject("baseline"), locale = "es") as JSONObject
        val approvalAlert = alerted.getJSONArray("alerts").getJSONObject(0)
        check(approvalAlert.getString("group") == "attention" && approvalAlert.getString("body") == "Aprobación: omarchy-t3-code" && alerted.getJSONObject("card").getJSONObject("chip").getString("kind") == "approve") { alerted.toString() }
        val seenAgain = api.domain.awareness.view(askEnvironments, 1792036860000L, baseline = alerted.getJSONObject("baseline")) as JSONObject
        check(seenAgain.getJSONArray("alerts").length() == 0 && seenAgain.getLong("nextDeadline") == 1792036800000L + 24 * 60 * 60 * 1000L) { seenAgain.toString() }
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
        // The presentation modules of the other screens: diff, usage, pull requests, Git, settings, rewind, plans.
        val patch = "diff --git a/src/a.rs b/src/a.rs\n--- a/src/a.rs\n+++ b/src/a.rs\n@@ -1 +1 @@\n-old\n+new\n"
        val diff = api.domain.diffReview.present(JSONObject().put("diff", patch)) as JSONObject
        check(diff.getJSONArray("files").getJSONObject(0).getString("path") == "src/a.rs" && diff.getInt("additions") == 1) { diff.toString() }
        val usage = api.domain.usageSummary.request("week", "2026-10-20", "America/Santiago") as JSONObject
        check(usage.getJSONObject("payload").getString("sinceDay") == "2026-10-14" && usage.getString("untilDay") == "2026-10-20")
        val window = JSONObject().put("id", "5h").put("kind", "session").put("label", "5 hours").put("usedPercent", 42)
        val limits = api.domain.quotaSources.limits(JSONObject().put("checkedAt", "2026-10-18T12:00:00Z").put("windows", org.json.JSONArray().put(window))) as JSONObject
        check(limits.getJSONArray("windows").getJSONObject(0).getInt("usedPercent") == 42)
        val reference = JSONObject().put("projectId", "p").put("repository", "o/r").put("host", "github.com").put("number", 7)
        val comment = api.domain.pullRequests.payload("comment", reference, body = "Looks good") as JSONObject
        check(comment.getString("method") == "pullRequests.comment" && comment.getJSONObject("payload").getInt("number") == 7) { comment.toString() }
        val mismatch = api.domain.branchPicker.mismatch(threadBranch = "main", currentBranch = "dev") as JSONObject
        check(mismatch.getBoolean("canRestore"))
        check(api.domain.branchPicker.sanitize("  my new\tbranch ") == "my-new-branch")
        val status = JSONObject().put("isRepo", true).put("hasPrimaryRemote", true).put("isDefaultRef", false).put("refName", "feature/x")
            .put("hasWorkingTreeChanges", true).put("hasUpstream", true).put("aheadCount", 1).put("behindCount", 0)
        val gitActions = api.domain.gitControls.actions(status) as JSONObject
        check(gitActions.getBoolean("commit") && gitActions.getBoolean("push") && !gitActions.getBoolean("pull"))
        val inspector = JSONObject().put("sourceThreadId", "thread:source").put("sourceItemId", "item:1").put("type", "command_execution")
            .put("runId", "run:1").put("providerTurnId", "turn:1").put("revision", "2026-10-17T12:00:01.000Z")
        val read = api.domain.toolInspector.request(inspector, "laptop", "thread:viewer") as JSONObject
        check(read.getString("method") == "orchestration.getTurnItem" && read.getJSONObject("payload").getString("itemId") == "item:1")
        val plan = JSONObject().put("id", "plan-1").put("threadId", "thread-1").put("status", "active").put("markdown", "# Ship it\n\n1. Build")
        val implement = api.domain.planWorkflow.followUp(plan, "thread-1") as JSONObject
        check(implement.getString("interactionMode") == "default" && implement.getJSONObject("sourcePlanRef").getString("planId") == "plan-1")
        check(api.domain.planWorkflow.implementationTitle("# Ship it") == "Implement Ship it")
        val setting = api.domain.nativeSettings.patch(JSONObject().put("key", "defaultRuntimeMode").put("value", "auto")) as JSONObject
        check(setting.getJSONObject("patch").getString("defaultRuntimeMode") == "auto")
        val projectFile = api.domain.scopedSettings.projectFile("{// note\n\"worktreeSubmodules\":\"none\",}") as JSONObject
        check(projectFile.getString("worktreeSubmodules") == "none")
        val gate = api.domain.checkpointRevert.safety(JSONObject().put("canOperate", true)) as JSONObject
        check(!gate.getBoolean("allowed") && gate.getString("error") == "revert_disconnected") { gate.toString() }
        // The ports: minimap, subagents, theme colours and ot3's string catalog.
        val rows = org.json.JSONArray().put(JSONObject().put("id", "u1").put("role", "user").put("text", "Fix\n the build"))
            .put(JSONObject().put("id", "a1").put("role", "assistant").put("text", "Done."))
            .put(JSONObject().put("id", "u2").put("role", "user").put("text", "Thanks"))
        val minimap = api.domain.timeline.minimap(rows, atEnd = true) as JSONObject
        check(minimap.getBoolean("visible") && minimap.getInt("currentIndex") == 1 && minimap.getJSONArray("items").getJSONObject(0).getString("userPreview") == "Fix the build")
        check((api.domain.timeline.minimapIndex(75.0, 100.0, 3) as Number).toInt() == 2)
        val cards = api.domain.subagents.cards(org.json.JSONArray().put(JSONObject().put("id", "s").put("status", "running").put("startedAt", 1000)), 250000) as JSONObject
        val card = cards.getJSONArray("subagents").getJSONObject(0).getJSONObject("view")
        check(card.getString("label") == "Running" && card.getString("elapsed") == "4m 09s" && card.getString("tone") == "working") { cards.toString() }
        val tokens = api.domain.theme.tokens("background = \"#1a1b26\"\nforeground = \"#a9b1d6\"\naccent = \"#7aa2f7\"\n", "tokyo-night") as JSONObject
        check(tokens.getString("name") == "Tokyo Night" && tokens.getString("appearance") == "dark" && tokens.getJSONObject("roles").getString("dim") == "#6b718a") { tokens.toString() }
        check(api.domain.i18n.t("es", "Close") == "Cerrar")
        check(api.domain.i18n.t("de", "{n} subagents", JSONObject().put("n", 3)) == "3 Unteragenten")
        val catalog = api.domain.i18n.catalog("fr_CA.UTF-8") as JSONObject
        check(catalog.getString("language") == "fr" && catalog.getJSONObject("strings").length() == 1804)
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
