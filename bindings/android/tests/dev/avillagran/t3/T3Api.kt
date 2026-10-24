// Generated from bindings/android/describe.json (t3code@680152e27) by tests/api_describe.rs.
// Do not edit: run `T3_UPDATE_FIXTURES=1 cargo test --test api_describe`.
package dev.avillagran.t3

import org.json.JSONArray
import org.json.JSONObject

/** Typed wrappers over [T3Client.call] and [T3Client.subscribeFn], one class per `describe` module. */
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
        /** `device.subscribeDeviceState` · stream · read · replay free · scope orchestration:read */
        fun subscribeDeviceState(): T3Subscription =
            client.subscribeFn("device.subscribeDeviceState", JSONObject())
    }

    class Diagnostics internal constructor(private val client: T3Client) {
        /** `server.getStorageCleanupReport` · stream · read · replay free · scope orchestration:read */
        fun getStorageCleanupReport(): T3Subscription =
            client.subscribeFn("server.getStorageCleanupReport", JSONObject())
        /** `diagnostics.storageCleanupReport` · read · replay free · scope orchestration:read · 10000 ms */
        fun storageCleanupReport(): Any? =
            client.call("diagnostics.storageCleanupReport", JSONObject())
    }

    class Orchestration internal constructor(private val client: T3Client) {
        /** `orchestration.dispatchCommand` · mutation · replay id · scope orchestration:operate · 20000 ms */
        fun dispatchCommand(args: JSONObject): Any? = client.call("orchestration.dispatchCommand", args)
        /** `orchestration.launchThread` · mutation · replay id · scope orchestration:operate · 20000 ms */
        fun launchThread(commandId: String, projectId: String, title: String, modelSelection: JSONObject, runtimeMode: String, interactionMode: String, workspaceStrategy: JSONObject, creationSource: String? = null, threadId: String? = null, reuseExistingThread: Boolean? = null, generateTitle: Boolean? = null, initialMessage: JSONObject? = null): Any? =
            client.call("orchestration.launchThread", JSONObject().put("commandId", commandId).put("projectId", projectId).put("title", title).put("modelSelection", modelSelection).put("runtimeMode", runtimeMode).put("interactionMode", interactionMode).put("workspaceStrategy", workspaceStrategy).put("creationSource", creationSource).put("threadId", threadId).put("reuseExistingThread", reuseExistingThread).put("generateTitle", generateTitle).put("initialMessage", initialMessage))
        /** `orchestration.subscribeShell` · stream · read · replay free · scope orchestration:read */
        fun subscribeShell(afterSequence: Long? = null, requestCompletionMarker: Boolean? = null): T3Subscription =
            client.subscribeFn("orchestration.subscribeShell", JSONObject().put("afterSequence", afterSequence).put("requestCompletionMarker", requestCompletionMarker))
        /** `orchestration.subscribeThread` · stream · read · replay free · scope orchestration:read */
        fun subscribeThread(threadId: String, afterSequence: Long? = null, requestCompletionMarker: Boolean? = null, acceptBoundedSnapshot: Boolean? = null, acceptCompactTurnItems: Boolean? = null): T3Subscription =
            client.subscribeFn("orchestration.subscribeThread", JSONObject().put("threadId", threadId).put("afterSequence", afterSequence).put("requestCompletionMarker", requestCompletionMarker).put("acceptBoundedSnapshot", acceptBoundedSnapshot).put("acceptCompactTurnItems", acceptCompactTurnItems))
        /** `orchestration.subscribeCommandOutput` · stream · read · replay free · scope orchestration:read */
        fun subscribeCommandOutput(threadId: String, itemId: String): T3Subscription =
            client.subscribeFn("orchestration.subscribeCommandOutput", JSONObject().put("threadId", threadId).put("itemId", itemId))
    }

    class Preview internal constructor(private val client: T3Client) {
        /** `preview.list` · read · replay free · scope orchestration:read · 20000 ms */
        fun list(threadId: String): Any? =
            client.call("preview.list", JSONObject().put("threadId", threadId))
        /** `preview.subscribeDiscoveredLocalServers` · stream · read · replay free · scope orchestration:read */
        fun subscribeDiscoveredLocalServers(configuredUrls: JSONArray? = null): T3Subscription =
            client.subscribeFn("preview.subscribeDiscoveredLocalServers", JSONObject().put("configuredUrls", configuredUrls))
        /** `preview.subscribePreviewEvents` · stream · read · replay free · scope orchestration:read */
        fun subscribePreviewEvents(): T3Subscription =
            client.subscribeFn("preview.subscribePreviewEvents", JSONObject())
        /** `preview.discoveredServers` · read · replay free · scope orchestration:read · 4000 ms */
        fun discoveredServers(configuredUrls: JSONArray? = null): Any? =
            client.call("preview.discoveredServers", JSONObject().put("configuredUrls", configuredUrls))
    }

    class Projects internal constructor(private val client: T3Client) {
        /** `projects.readFile` · read · replay free · scope filesystem:read · 20000 ms */
        fun readFile(cwd: String, relativePath: String, allowMissing: Boolean? = null): Any? =
            client.call("projects.readFile", JSONObject().put("cwd", cwd).put("relativePath", relativePath).put("allowMissing", allowMissing))
    }

    class Provider internal constructor(private val client: T3Client) {
        /** `provider.install.subscribe` · stream · read · replay free · scope orchestration:read */
        fun installSubscribe(instanceId: String): T3Subscription =
            client.subscribeFn("provider.install.subscribe", JSONObject().put("instanceId", instanceId))
        /** `provider.auth.subscribe` · stream · read · replay free · scope providers:manage */
        fun authSubscribe(instanceId: String): T3Subscription =
            client.subscribeFn("provider.auth.subscribe", JSONObject().put("instanceId", instanceId))
        /** `provider.installState` · read · replay free · scope orchestration:read · 10000 ms */
        fun installState(instanceId: String): Any? =
            client.call("provider.installState", JSONObject().put("instanceId", instanceId))
        /** `provider.authState` · read · replay free · scope providers:manage · 10000 ms */
        fun authState(instanceId: String): Any? =
            client.call("provider.authState", JSONObject().put("instanceId", instanceId))
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
        /** `server.subscribeServerConfig` · stream · read · replay free · scope orchestration:read */
        fun subscribeServerConfig(environmentThemes: Boolean? = null, usageLimitSources: Boolean? = null, usageLimitsCommand: Boolean? = null): T3Subscription =
            client.subscribeFn("server.subscribeServerConfig", JSONObject().put("environmentThemes", environmentThemes).put("usageLimitSources", usageLimitSources).put("usageLimitsCommand", usageLimitsCommand))
    }

    class Terminal internal constructor(private val client: T3Client) {
        /** `terminal.attach` · stream · mutation · replay set · scope terminal:operate */
        fun attach(threadId: String, terminalId: String, cwd: String? = null, worktreePath: String? = null, cols: Long? = null, rows: Long? = null, env: JSONObject? = null, providerInstanceId: String? = null, restartIfNotRunning: Boolean? = null, resume: Boolean? = null): T3Subscription =
            client.subscribeFn("terminal.attach", JSONObject().put("threadId", threadId).put("terminalId", terminalId).put("cwd", cwd).put("worktreePath", worktreePath).put("cols", cols).put("rows", rows).put("env", env).put("providerInstanceId", providerInstanceId).put("restartIfNotRunning", restartIfNotRunning).put("resume", resume))
    }

    class Vcs internal constructor(private val client: T3Client) {
        /** `vcs.refreshStatus` · read · replay free · scope orchestration:read · 20000 ms */
        fun refreshStatus(cwd: String): Any? =
            client.call("vcs.refreshStatus", JSONObject().put("cwd", cwd))
        /** `git.runStackedAction` · stream · mutation · replay never · scope one of source-control:write, orchestration:read */
        fun runStackedAction(actionId: String, cwd: String, action: String, commitMessage: String? = null, featureBranch: Boolean? = null, filePaths: JSONArray? = null, threadId: String? = null, projectId: String? = null): T3Subscription =
            client.subscribeFn("git.runStackedAction", JSONObject().put("actionId", actionId).put("cwd", cwd).put("action", action).put("commitMessage", commitMessage).put("featureBranch", featureBranch).put("filePaths", filePaths).put("threadId", threadId).put("projectId", projectId))
    }

    class WorktreeSetup internal constructor(private val client: T3Client) {
        /** `worktreeSetup.subscribeWorktreeSetup` · stream · read · replay free · scope orchestration:read */
        fun subscribeWorktreeSetup(threadId: String): T3Subscription =
            client.subscribeFn("worktreeSetup.subscribeWorktreeSetup", JSONObject().put("threadId", threadId))
    }
}
