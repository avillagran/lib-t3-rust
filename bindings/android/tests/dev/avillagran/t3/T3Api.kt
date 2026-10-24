// Generated from bindings/android/describe.json (t3code@680152e27) by tests/api_describe.rs.
// Do not edit: run `T3_UPDATE_FIXTURES=1 cargo test --test api_describe`.
package dev.avillagran.t3

import org.json.JSONArray
import org.json.JSONObject

/** Typed wrappers over [T3Client.call], one class per `describe` module. */
@Suppress("unused")
class T3Api(client: T3Client) {
    val assets = Assets(client)
    val device = Device(client)
    val diagnostics = Diagnostics(client)
    val orchestration = Orchestration(client)
    val preview = Preview(client)
    val projects = Projects(client)
    val provider = Provider(client)
    val review = Review(client)
    val server = Server(client)
    val terminal = Terminal(client)
    val vcs = Vcs(client)
    val worktreeSetup = WorktreeSetup(client)

    class Assets internal constructor(private val client: T3Client) {
        /** `assets.createUrl` · read · replay free · scope one of filesystem:read, orchestration:read · 20000 ms */
        fun createUrl(resource: JSONObject): Any? =
            client.call("assets.createUrl", JSONObject().put("resource", resource))
    }

    class Device internal constructor(private val client: T3Client) {
    }

    class Diagnostics internal constructor(private val client: T3Client) {
    }

    class Orchestration internal constructor(private val client: T3Client) {
        /** `orchestration.dispatchCommand` · mutation · replay id · scope orchestration:operate · 20000 ms */
        fun dispatchCommand(args: JSONObject): Any? = client.call("orchestration.dispatchCommand", args)
        /** `orchestration.launchThread` · mutation · replay id · scope orchestration:operate · 20000 ms */
        fun launchThread(commandId: String, projectId: String, title: String, modelSelection: JSONObject, runtimeMode: String, interactionMode: String, workspaceStrategy: JSONObject, creationSource: String? = null, threadId: String? = null, reuseExistingThread: Boolean? = null, generateTitle: Boolean? = null, initialMessage: JSONObject? = null): Any? =
            client.call("orchestration.launchThread", JSONObject().put("commandId", commandId).put("projectId", projectId).put("title", title).put("modelSelection", modelSelection).put("runtimeMode", runtimeMode).put("interactionMode", interactionMode).put("workspaceStrategy", workspaceStrategy).put("creationSource", creationSource).put("threadId", threadId).put("reuseExistingThread", reuseExistingThread).put("generateTitle", generateTitle).put("initialMessage", initialMessage))
    }

    class Preview internal constructor(private val client: T3Client) {
        /** `preview.list` · read · replay free · scope orchestration:read · 20000 ms */
        fun list(threadId: String): Any? =
            client.call("preview.list", JSONObject().put("threadId", threadId))
    }

    class Projects internal constructor(private val client: T3Client) {
        /** `projects.readFile` · read · replay free · scope filesystem:read · 20000 ms */
        fun readFile(cwd: String, relativePath: String, allowMissing: Boolean? = null): Any? =
            client.call("projects.readFile", JSONObject().put("cwd", cwd).put("relativePath", relativePath).put("allowMissing", allowMissing))
    }

    class Provider internal constructor(private val client: T3Client) {
    }

    class Review internal constructor(private val client: T3Client) {
        /** `review.getDiffPreview` · read · replay free · scope filesystem:read · 20000 ms */
        fun getDiffPreview(cwd: String, baseRef: String? = null, ignoreWhitespace: Boolean? = null, file: JSONObject? = null): Any? =
            client.call("review.getDiffPreview", JSONObject().put("cwd", cwd).put("baseRef", baseRef).put("ignoreWhitespace", ignoreWhitespace).put("file", file))
    }

    class Server internal constructor(private val client: T3Client) {
        /** `server.getConfig` · read · replay free · scope orchestration:read · 20000 ms */
        fun getConfig(): Any? =
            client.call("server.getConfig", JSONObject())
        /** `server.getSettings` · read · replay free · scope orchestration:read · 20000 ms */
        fun getSettings(): Any? =
            client.call("server.getSettings", JSONObject())
        /** `server.updateSettings` · mutation · replay set · scope one of settings:write, providers:manage · 20000 ms */
        fun updateSettings(patch: JSONObject, providerInstanceMutation: JSONObject? = null): Any? =
            client.call("server.updateSettings", JSONObject().put("patch", patch).put("providerInstanceMutation", providerInstanceMutation))
    }

    class Terminal internal constructor(private val client: T3Client) {
    }

    class Vcs internal constructor(private val client: T3Client) {
        /** `vcs.refreshStatus` · read · replay free · scope orchestration:read · 20000 ms */
        fun refreshStatus(cwd: String): Any? =
            client.call("vcs.refreshStatus", JSONObject().put("cwd", cwd))
    }

    class WorktreeSetup internal constructor(private val client: T3Client) {
    }
}
