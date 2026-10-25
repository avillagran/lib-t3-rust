// Generated from bindings/android/describe.json (t3code@680152e27) by tests/api_describe.rs.
// Do not edit: run `T3_UPDATE_FIXTURES=1 cargo test --test api_describe`.
package dev.avillagran.t3

import org.json.JSONArray
import org.json.JSONObject

/** Typed wrappers over [T3Client.call] and [T3Client.subscribeFn], one class per `describe` module;
 *  [domain] wraps [T3Client.domain], one class per `describe` domain module. */
@Suppress("unused")
class T3Api(client: T3Client) {
    val agentSessions = AgentSessions(client)
    val assets = Assets(client)
    val device = Device(client)
    val diagnostics = Diagnostics(client)
    val filesystem = Filesystem(client)
    val mcpApps = McpApps(client)
    val orchestration = Orchestration(client)
    val preview = Preview(client)
    val projectClone = ProjectClone(client)
    val projects = Projects(client)
    val provider = Provider(client)
    val pullRequests = PullRequests(client)
    val review = Review(client)
    val scheduledTasks = ScheduledTasks(client)
    val secrets = Secrets(client)
    val server = Server(client)
    val shell = Shell(client)
    val terminal = Terminal(client)
    val vcs = Vcs(client)
    val worktreeSetup = WorktreeSetup(client)
    val domain = Domain(client)

    class AgentSessions internal constructor(private val client: T3Client) {
        /** `agentSessions.scan` · read · replay free · scope orchestration:read · 20000 ms */
        fun scan(): Any? =
            client.call("agentSessions.scan", JSONObject())
        /** `agentSessions.import` · mutation · replay set · scope orchestration:operate · 20000 ms */
        fun import(projectId: String, expectedWorkspaceRoot: String? = null): Any? =
            client.call("agentSessions.import", JSONObject().put("projectId", projectId).put("expectedWorkspaceRoot", expectedWorkspaceRoot))
    }

    class Assets internal constructor(private val client: T3Client) {
        /** `assets.createUrl` · read · replay free · scope one of filesystem:read, orchestration:read · 20000 ms */
        fun createUrl(resource: JSONObject): Any? =
            client.call("assets.createUrl", JSONObject().put("resource", resource))
        /** `attachments.delete` · mutation · replay set · scope orchestration:operate · 20000 ms */
        fun delete(attachmentId: String): Any? =
            client.call("attachments.delete", JSONObject().put("attachmentId", attachmentId))
    }

    class Device internal constructor(private val client: T3Client) {
        /** `device.list` · read · replay free · scope one of orchestration:read, orchestration:operate · 300000 ms */
        fun list(updateTool: String? = null, inspectOnly: Boolean? = null, retryHostId: String? = null): Any? =
            client.call("device.list", JSONObject().put("updateTool", updateTool).put("inspectOnly", inspectOnly).put("retryHostId", retryHostId))
        /** `device.configure` · mutation · replay set · scope settings:write · 300000 ms */
        fun configure(enabled: Boolean? = null, agentAccessEnabled: Boolean? = null, onboardingCompleted: Boolean? = null): Any? =
            client.call("device.configure", JSONObject().put("enabled", enabled).put("agentAccessEnabled", agentAccessEnabled).put("onboardingCompleted", onboardingCompleted))
        /** `device.testHost` · read · replay free · scope settings:write · 300000 ms */
        fun testHost(id: String, label: String, target: String, kind: String? = null, identityFile: String? = null, port: Long? = null): Any? =
            client.call("device.testHost", JSONObject().put("id", id).put("label", label).put("target", target).put("kind", kind).put("identityFile", identityFile).put("port", port))
        /** `device.open` · mutation · replay never · scope orchestration:operate · 180000 ms */
        fun open(threadId: String, deviceId: String, platform: String, hostId: String? = null, boot: Boolean? = null): Any? =
            client.call("device.open", JSONObject().put("threadId", threadId).put("deviceId", deviceId).put("platform", platform).put("hostId", hostId).put("boot", boot))
        /** `device.close` · mutation · replay set · scope orchestration:operate · 60000 ms */
        fun close(threadId: String, hostId: String? = null, deviceId: String? = null, shutdown: Boolean? = null): Any? =
            client.call("device.close", JSONObject().put("threadId", threadId).put("hostId", hostId).put("deviceId", deviceId).put("shutdown", shutdown))
        /** `device.detail` · read · replay free · scope orchestration:read · 20000 ms */
        fun detail(deviceId: String, hostId: String? = null): Any? =
            client.call("device.detail", JSONObject().put("deviceId", deviceId).put("hostId", hostId))
        /** `device.action` · mutation · replay never · scope orchestration:operate · 30000 ms */
        fun action(args: JSONObject): Any? = client.call("device.action", args)
        /** `device.subscribeDeviceState` · stream · read · replay free · scope orchestration:read */
        fun subscribeDeviceState(): T3Subscription =
            client.subscribeFn("device.subscribeDeviceState", JSONObject())
    }

    class Diagnostics internal constructor(private val client: T3Client) {
        /** `server.getHostResources` · read · replay free · scope orchestration:read · 20000 ms */
        fun getHostResources(): Any? =
            client.call("server.getHostResources", JSONObject())
        /** `server.getProcessDiagnostics` · read · replay free · scope diagnostics:read · 20000 ms */
        fun getProcessDiagnostics(): Any? =
            client.call("server.getProcessDiagnostics", JSONObject())
        /** `server.getResourceTelemetryHistory` · read · replay free · scope diagnostics:read · 20000 ms */
        fun getResourceTelemetryHistory(windowMs: Long, bucketMs: Long): Any? =
            client.call("server.getResourceTelemetryHistory", JSONObject().put("windowMs", windowMs).put("bucketMs", bucketMs))
        /** `server.signalProcess` · mutation · replay set · scope environment:maintain · 20000 ms */
        fun signalProcess(pid: Long, startTimeMs: Long, signal: String): Any? =
            client.call("server.signalProcess", JSONObject().put("pid", pid).put("startTimeMs", startTimeMs).put("signal", signal))
        /** `server.checkOtlpEndpoint` · read · replay free · scope settings:write · 20000 ms */
        fun checkOtlpEndpoint(signal: String, url: String): Any? =
            client.call("server.checkOtlpEndpoint", JSONObject().put("signal", signal).put("url", url))
        /** `server.getUsageSummary` · read · replay free · scope diagnostics:read · 20000 ms */
        fun getUsageSummary(sinceDay: String, untilDay: String, timeZone: String, resolution: String? = null, sinceTime: String? = null, untilTime: String? = null, awaitRefresh: Boolean? = null): Any? =
            client.call("server.getUsageSummary", JSONObject().put("sinceDay", sinceDay).put("untilDay", untilDay).put("timeZone", timeZone).put("resolution", resolution).put("sinceTime", sinceTime).put("untilTime", untilTime).put("awaitRefresh", awaitRefresh))
        /** `server.refreshUsageRates` · read · replay free · scope diagnostics:read · 20000 ms */
        fun refreshUsageRates(): Any? =
            client.call("server.refreshUsageRates", JSONObject())
        /** `server.runStorageCleanup` · mutation · replay never · scope settings:write · 20000 ms */
        fun runStorageCleanup(): Any? =
            client.call("server.runStorageCleanup", JSONObject())
        /** `server.getStorageCleanupReport` · stream · read · replay free · scope orchestration:read */
        fun getStorageCleanupReport(): T3Subscription =
            client.subscribeFn("server.getStorageCleanupReport", JSONObject())
        /** `diagnostics.storageCleanupReport` · read · replay free · scope orchestration:read · 10000 ms */
        fun storageCleanupReport(): Any? =
            client.call("diagnostics.storageCleanupReport", JSONObject())
    }

    class Filesystem internal constructor(private val client: T3Client) {
        /** `filesystem.browse` · read · replay free · scope filesystem:read · 20000 ms */
        fun browse(partialPath: String, cwd: String? = null): Any? =
            client.call("filesystem.browse", JSONObject().put("partialPath", partialPath).put("cwd", cwd))
        /** `filesystem.getMetadata` · read · replay free · scope filesystem:read · 20000 ms */
        fun getMetadata(paths: JSONArray): Any? =
            client.call("filesystem.getMetadata", JSONObject().put("paths", paths))
    }

    class McpApps internal constructor(private val client: T3Client) {
        /** `mcpApps.callTool` · mutation · replay never · scope orchestration:operate · 20000 ms */
        fun callTool(threadId: String, itemId: String, name: String, arguments: JSONObject): Any? =
            client.call("mcpApps.callTool", JSONObject().put("threadId", threadId).put("itemId", itemId).put("name", name).put("arguments", arguments))
        /** `mcpApps.readResource` · read · replay free · scope orchestration:read · 20000 ms */
        fun readResource(threadId: String, itemId: String, uri: String): Any? =
            client.call("mcpApps.readResource", JSONObject().put("threadId", threadId).put("itemId", itemId).put("uri", uri))
        /** `mcpApps.updateModelContext` · mutation · replay set · scope orchestration:operate · 20000 ms */
        fun updateModelContext(threadId: String, itemId: String, conversationThreadId: String, content: JSONArray? = null, structuredContent: JSONObject? = null): Any? =
            client.call("mcpApps.updateModelContext", JSONObject().put("threadId", threadId).put("itemId", itemId).put("conversationThreadId", conversationThreadId).put("content", content).put("structuredContent", structuredContent))
    }

    class Orchestration internal constructor(private val client: T3Client) {
        /** `orchestration.dispatchCommand` · mutation · replay id · scope orchestration:operate · 20000 ms */
        fun dispatchCommand(args: JSONObject): Any? = client.call("orchestration.dispatchCommand", args)
        /** `orchestration.launchThread` · mutation · replay id · scope orchestration:operate · 20000 ms */
        fun launchThread(commandId: String, projectId: String, title: String, modelSelection: JSONObject, runtimeMode: String, interactionMode: String, workspaceStrategy: JSONObject, creationSource: String? = null, threadId: String? = null, reuseExistingThread: Boolean? = null, generateTitle: Boolean? = null, initialMessage: JSONObject? = null): Any? =
            client.call("orchestration.launchThread", JSONObject().put("commandId", commandId).put("projectId", projectId).put("title", title).put("modelSelection", modelSelection).put("runtimeMode", runtimeMode).put("interactionMode", interactionMode).put("workspaceStrategy", workspaceStrategy).put("creationSource", creationSource).put("threadId", threadId).put("reuseExistingThread", reuseExistingThread).put("generateTitle", generateTitle).put("initialMessage", initialMessage))
        /** `orchestration.getThreadProjection` · read · replay free · scope orchestration:read · 20000 ms */
        fun getThreadProjection(threadId: String): Any? =
            client.call("orchestration.getThreadProjection", JSONObject().put("threadId", threadId))
        /** `orchestration.getTurnItem` · read · replay free · scope orchestration:read · 20000 ms */
        fun getTurnItem(threadId: String, itemId: String, revision: String? = null): Any? =
            client.call("orchestration.getTurnItem", JSONObject().put("threadId", threadId).put("itemId", itemId).put("revision", revision))
        /** `orchestration.getTurnDiff` · read · replay free · scope orchestration:read · 20000 ms */
        fun getTurnDiff(threadId: String, fromTurnCount: Long, toTurnCount: Long, ignoreWhitespace: Boolean? = null): Any? =
            client.call("orchestration.getTurnDiff", JSONObject().put("threadId", threadId).put("fromTurnCount", fromTurnCount).put("toTurnCount", toTurnCount).put("ignoreWhitespace", ignoreWhitespace))
        /** `orchestration.getFullThreadDiff` · read · replay free · scope orchestration:read · 20000 ms */
        fun getFullThreadDiff(threadId: String, toTurnCount: Long, ignoreWhitespace: Boolean? = null): Any? =
            client.call("orchestration.getFullThreadDiff", JSONObject().put("threadId", threadId).put("toTurnCount", toTurnCount).put("ignoreWhitespace", ignoreWhitespace))
        /** `orchestration.getWorkflowScript` · read · replay free · scope orchestration:read · 20000 ms */
        fun getWorkflowScript(threadId: String, scriptPath: String): Any? =
            client.call("orchestration.getWorkflowScript", JSONObject().put("threadId", threadId).put("scriptPath", scriptPath))
        /** `orchestration.stopWorkflow` · mutation · replay set · scope orchestration:operate · 20000 ms */
        fun stopWorkflow(threadId: String, subagentId: String): Any? =
            client.call("orchestration.stopWorkflow", JSONObject().put("threadId", threadId).put("subagentId", subagentId))
        /** `orchestration.searchThreads` · read · replay free · scope orchestration:read · 20000 ms */
        fun searchThreads(query: String, limit: Long? = null): Any? =
            client.call("orchestration.searchThreads", JSONObject().put("query", query).put("limit", limit))
        /** `orchestration.getArchivedShellSnapshot` · read · replay free · scope orchestration:read · 20000 ms */
        fun getArchivedShellSnapshot(): Any? =
            client.call("orchestration.getArchivedShellSnapshot", JSONObject())
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
        /** `preview.open` · mutation · replay never · scope preview:operate · 20000 ms */
        fun open(threadId: String, url: String? = null, viewport: JSONObject? = null, profileId: String? = null, runtime: String? = null, reveal: Boolean? = null): Any? =
            client.call("preview.open", JSONObject().put("threadId", threadId).put("url", url).put("viewport", viewport).put("profileId", profileId).put("runtime", runtime).put("reveal", reveal))
        /** `preview.navigate` · mutation · replay set · scope preview:operate · 20000 ms */
        fun navigate(threadId: String, tabId: String, url: String, resolvedTitle: String? = null): Any? =
            client.call("preview.navigate", JSONObject().put("threadId", threadId).put("tabId", tabId).put("url", url).put("resolvedTitle", resolvedTitle))
        /** `preview.resize` · mutation · replay set · scope preview:operate · 20000 ms */
        fun resize(threadId: String, tabId: String, viewport: JSONObject): Any? =
            client.call("preview.resize", JSONObject().put("threadId", threadId).put("tabId", tabId).put("viewport", viewport))
        /** `preview.adjust` · mutation · replay set · scope preview:operate · 20000 ms */
        fun adjust(threadId: String, tabId: String, colorScheme: String? = null, zoomFactor: Double? = null, hardReload: Boolean? = null, clear: String? = null): Any? =
            client.call("preview.adjust", JSONObject().put("threadId", threadId).put("tabId", tabId).put("colorScheme", colorScheme).put("zoomFactor", zoomFactor).put("hardReload", hardReload).put("clear", clear))
        /** `preview.refresh` · mutation · replay set · scope preview:operate · 20000 ms */
        fun refresh(threadId: String, tabId: String): Any? =
            client.call("preview.refresh", JSONObject().put("threadId", threadId).put("tabId", tabId))
        /** `preview.close` · mutation · replay set · scope preview:operate · 20000 ms */
        fun close(threadId: String, tabId: String? = null): Any? =
            client.call("preview.close", JSONObject().put("threadId", threadId).put("tabId", tabId))
        /** `preview.clearProfile` · mutation · replay set · scope preview:operate · 60000 ms */
        fun clearProfile(profileId: String): Any? =
            client.call("preview.clearProfile", JSONObject().put("profileId", profileId))
        /** `preview.reportStatus` · mutation · replay set · scope preview:operate · 20000 ms */
        fun reportStatus(threadId: String, tabId: String, navStatus: JSONObject, canGoBack: Boolean, canGoForward: Boolean): Any? =
            client.call("preview.reportStatus", JSONObject().put("threadId", threadId).put("tabId", tabId).put("navStatus", navStatus).put("canGoBack", canGoBack).put("canGoForward", canGoForward))
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

    class ProjectClone internal constructor(private val client: T3Client) {
        /** `projectClone.start` · mutation · replay never · scope source-control:write · 20000 ms */
        fun start(projectId: String, title: String, createdAt: String, destinationPath: String, provider: String? = null, repository: String? = null, remoteUrl: String? = null, protocol: String? = null): Any? =
            client.call("projectClone.start", JSONObject().put("projectId", projectId).put("title", title).put("createdAt", createdAt).put("destinationPath", destinationPath).put("provider", provider).put("repository", repository).put("remoteUrl", remoteUrl).put("protocol", protocol))
    }

    class Projects internal constructor(private val client: T3Client) {
        /** `projects.readFile` · read · replay free · scope filesystem:read · 20000 ms */
        fun readFile(cwd: String, relativePath: String, allowMissing: Boolean? = null): Any? =
            client.call("projects.readFile", JSONObject().put("cwd", cwd).put("relativePath", relativePath).put("allowMissing", allowMissing))
        /** `projects.searchEntries` · read · replay free · scope filesystem:read · 20000 ms */
        fun searchEntries(cwd: String, query: String, limit: Long, kind: String? = null, imageOnly: Boolean? = null): Any? =
            client.call("projects.searchEntries", JSONObject().put("cwd", cwd).put("query", query).put("limit", limit).put("kind", kind).put("imageOnly", imageOnly))
        /** `projects.mutate` · mutation · replay id · scope one of orchestration:operate, settings:write · 20000 ms */
        fun mutate(args: JSONObject): Any? = client.call("projects.mutate", args)
        /** `projects.createNew` · mutation · replay never · scope orchestration:operate · 20000 ms */
        fun createNew(name: String): Any? =
            client.call("projects.createNew", JSONObject().put("name", name))
    }

    class Provider internal constructor(private val client: T3Client) {
        /** `server.refreshProviders` · read · replay free · scope orchestration:read · 20000 ms */
        fun refreshProviders(instanceId: String? = null, cwd: String? = null, fresh: Boolean? = null, refreshModels: Boolean? = null): Any? =
            client.call("server.refreshProviders", JSONObject().put("instanceId", instanceId).put("cwd", cwd).put("fresh", fresh).put("refreshModels", refreshModels))
        /** `server.updateProvider` · mutation · replay never · scope providers:manage · 300000 ms */
        fun updateProvider(provider: String, targetVersion: String? = null, instanceId: String? = null): Any? =
            client.call("server.updateProvider", JSONObject().put("provider", provider).put("targetVersion", targetVersion).put("instanceId", instanceId))
        /** `provider.auth.start` · mutation · replay never · scope providers:manage · 20000 ms */
        fun authStart(instanceId: String, methodId: String? = null, returnUrl: String? = null, callbackMode: String? = null): Any? =
            client.call("provider.auth.start", JSONObject().put("instanceId", instanceId).put("methodId", methodId).put("returnUrl", returnUrl).put("callbackMode", callbackMode))
        /** `provider.auth.respond` · mutation · replay never · scope providers:manage · 20000 ms */
        fun authRespond(instanceId: String, flowId: String, interactionId: String, response: JSONObject): Any? =
            client.call("provider.auth.respond", JSONObject().put("instanceId", instanceId).put("flowId", flowId).put("interactionId", interactionId).put("response", response))
        /** `provider.auth.cancel` · mutation · replay set · scope providers:manage · 20000 ms */
        fun authCancel(instanceId: String, flowId: String): Any? =
            client.call("provider.auth.cancel", JSONObject().put("instanceId", instanceId).put("flowId", flowId))
        /** `provider.auth.logout` · mutation · replay set · scope providers:manage · 20000 ms */
        fun authLogout(instanceId: String): Any? =
            client.call("provider.auth.logout", JSONObject().put("instanceId", instanceId))
        /** `provider.install.start` · mutation · replay never · scope providers:manage · 20000 ms */
        fun installStart(instanceId: String): Any? =
            client.call("provider.install.start", JSONObject().put("instanceId", instanceId))
        /** `provider.install.cancel` · mutation · replay set · scope providers:manage · 20000 ms */
        fun installCancel(instanceId: String, operationId: String): Any? =
            client.call("provider.install.cancel", JSONObject().put("instanceId", instanceId).put("operationId", operationId))
        /** `provider.install.remove` · mutation · replay set · scope providers:manage · 20000 ms */
        fun installRemove(instanceId: String): Any? =
            client.call("provider.install.remove", JSONObject().put("instanceId", instanceId))
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

    class PullRequests internal constructor(private val client: T3Client) {
        /** `pullRequests.list` · read · replay free · scope orchestration:read · 20000 ms */
        fun list(state: String, involvement: String? = null, filters: JSONObject? = null, projectId: String? = null, projectIds: JSONArray? = null, host: String? = null, limit: Long? = null, cursors: JSONObject? = null, query: String? = null): Any? =
            client.call("pullRequests.list", JSONObject().put("state", state).put("involvement", involvement).put("filters", filters).put("projectId", projectId).put("projectIds", projectIds).put("host", host).put("limit", limit).put("cursors", cursors).put("query", query))
        /** `pullRequests.detail` · read · replay free · scope orchestration:read · 20000 ms */
        fun detail(projectId: String, repository: String, number: Long, host: String? = null, expectedAccountId: String? = null, allowStale: Boolean? = null): Any? =
            client.call("pullRequests.detail", JSONObject().put("projectId", projectId).put("repository", repository).put("number", number).put("host", host).put("expectedAccountId", expectedAccountId).put("allowStale", allowStale))
        /** `pullRequests.checks` · read · replay free · scope orchestration:read · 20000 ms */
        fun checks(projectId: String, repository: String, number: Long, host: String? = null, expectedAccountId: String? = null, allowStale: Boolean? = null): Any? =
            client.call("pullRequests.checks", JSONObject().put("projectId", projectId).put("repository", repository).put("number", number).put("host", host).put("expectedAccountId", expectedAccountId).put("allowStale", allowStale))
        /** `pullRequests.activity` · read · replay free · scope orchestration:read · 20000 ms */
        fun activity(projectId: String, repository: String, number: Long, host: String? = null, expectedAccountId: String? = null, allowStale: Boolean? = null): Any? =
            client.call("pullRequests.activity", JSONObject().put("projectId", projectId).put("repository", repository).put("number", number).put("host", host).put("expectedAccountId", expectedAccountId).put("allowStale", allowStale))
        /** `pullRequests.routing` · read · replay free · scope orchestration:read · 20000 ms */
        fun routing(projectId: String, repository: String, number: Long, host: String? = null, expectedAccountId: String? = null, allowStale: Boolean? = null): Any? =
            client.call("pullRequests.routing", JSONObject().put("projectId", projectId).put("repository", repository).put("number", number).put("host", host).put("expectedAccountId", expectedAccountId).put("allowStale", allowStale))
        /** `pullRequests.routingIdentity` · read · replay free · scope orchestration:read · 20000 ms */
        fun routingIdentity(host: String): Any? =
            client.call("pullRequests.routingIdentity", JSONObject().put("host", host))
        /** `pullRequests.reviewerCandidates` · read · replay free · scope orchestration:read · 20000 ms */
        fun reviewerCandidates(projectId: String, repository: String, number: Long, host: String? = null, expectedAccountId: String? = null, allowStale: Boolean? = null): Any? =
            client.call("pullRequests.reviewerCandidates", JSONObject().put("projectId", projectId).put("repository", repository).put("number", number).put("host", host).put("expectedAccountId", expectedAccountId).put("allowStale", allowStale))
        /** `pullRequests.labelCandidates` · read · replay free · scope orchestration:read · 20000 ms */
        fun labelCandidates(projectId: String, repository: String, number: Long, host: String? = null, expectedAccountId: String? = null, allowStale: Boolean? = null): Any? =
            client.call("pullRequests.labelCandidates", JSONObject().put("projectId", projectId).put("repository", repository).put("number", number).put("host", host).put("expectedAccountId", expectedAccountId).put("allowStale", allowStale))
        /** `pullRequests.comment` · mutation · replay never · scope source-control:write · 20000 ms */
        fun comment(projectId: String, repository: String, number: Long, body: String, host: String? = null, expectedAccountId: String? = null, allowStale: Boolean? = null): Any? =
            client.call("pullRequests.comment", JSONObject().put("projectId", projectId).put("repository", repository).put("number", number).put("body", body).put("host", host).put("expectedAccountId", expectedAccountId).put("allowStale", allowStale))
        /** `pullRequests.submitReview` · mutation · replay never · scope source-control:write · 20000 ms */
        fun submitReview(projectId: String, repository: String, number: Long, verdict: String, body: String, comments: JSONArray, host: String? = null, expectedAccountId: String? = null, allowStale: Boolean? = null): Any? =
            client.call("pullRequests.submitReview", JSONObject().put("projectId", projectId).put("repository", repository).put("number", number).put("verdict", verdict).put("body", body).put("comments", comments).put("host", host).put("expectedAccountId", expectedAccountId).put("allowStale", allowStale))
        /** `pullRequests.runAction` · mutation · replay never · scope source-control:write · 20000 ms */
        fun runAction(projectId: String, repository: String, number: Long, action: String, mergeMethod: String? = null, updateMethod: String? = null, stackNumber: Long? = null, expectedStackHeads: JSONArray? = null, host: String? = null, expectedAccountId: String? = null, allowStale: Boolean? = null): Any? =
            client.call("pullRequests.runAction", JSONObject().put("projectId", projectId).put("repository", repository).put("number", number).put("action", action).put("mergeMethod", mergeMethod).put("updateMethod", updateMethod).put("stackNumber", stackNumber).put("expectedStackHeads", expectedStackHeads).put("host", host).put("expectedAccountId", expectedAccountId).put("allowStale", allowStale))
        /** `pullRequests.requestReviewers` · mutation · replay set · scope source-control:write · 20000 ms */
        fun requestReviewers(projectId: String, repository: String, number: Long, reviewers: JSONArray, requested: Boolean, host: String? = null, expectedAccountId: String? = null, allowStale: Boolean? = null): Any? =
            client.call("pullRequests.requestReviewers", JSONObject().put("projectId", projectId).put("repository", repository).put("number", number).put("reviewers", reviewers).put("requested", requested).put("host", host).put("expectedAccountId", expectedAccountId).put("allowStale", allowStale))
        /** `pullRequests.setLabels` · mutation · replay set · scope source-control:write · 20000 ms */
        fun setLabels(projectId: String, repository: String, number: Long, labels: JSONArray, applied: Boolean, host: String? = null, expectedAccountId: String? = null, allowStale: Boolean? = null): Any? =
            client.call("pullRequests.setLabels", JSONObject().put("projectId", projectId).put("repository", repository).put("number", number).put("labels", labels).put("applied", applied).put("host", host).put("expectedAccountId", expectedAccountId).put("allowStale", allowStale))
    }

    class Review internal constructor(private val client: T3Client) {
        /** `review.getDiffPreview` · read · replay free · scope filesystem:read · 20000 ms */
        fun getDiffPreview(cwd: String, baseRef: String? = null, ignoreWhitespace: Boolean? = null, file: JSONObject? = null): Any? =
            client.call("review.getDiffPreview", JSONObject().put("cwd", cwd).put("baseRef", baseRef).put("ignoreWhitespace", ignoreWhitespace).put("file", file))
        /** `review.getDiffFileContents` · read · replay free · scope filesystem:read · 20000 ms */
        fun getDiffFileContents(cwd: String, sourceKind: String, changeType: String, oldPath: String, newPath: String, baseRef: String? = null, headRef: String? = null): Any? =
            client.call("review.getDiffFileContents", JSONObject().put("cwd", cwd).put("sourceKind", sourceKind).put("changeType", changeType).put("oldPath", oldPath).put("newPath", newPath).put("baseRef", baseRef).put("headRef", headRef))
    }

    class ScheduledTasks internal constructor(private val client: T3Client) {
        /** `scheduledTasks.list` · read · replay free · scope orchestration:read · 20000 ms */
        fun list(): Any? =
            client.call("scheduledTasks.list", JSONObject())
        /** `scheduledTasks.upsert` · mutation · replay id · scope orchestration:operate · 20000 ms */
        fun upsert(title: String, prompt: String, enabled: Boolean, schedule: JSONObject, projectId: String, workspaceStrategy: JSONObject, modelSelection: JSONObject, runtimeMode: String, interactionMode: String, commandId: String? = null, id: String? = null, requireExisting: Boolean? = null, threadId: String? = null, createdBy: String? = null, creationSource: String? = null): Any? =
            client.call("scheduledTasks.upsert", JSONObject().put("title", title).put("prompt", prompt).put("enabled", enabled).put("schedule", schedule).put("projectId", projectId).put("workspaceStrategy", workspaceStrategy).put("modelSelection", modelSelection).put("runtimeMode", runtimeMode).put("interactionMode", interactionMode).put("commandId", commandId).put("id", id).put("requireExisting", requireExisting).put("threadId", threadId).put("createdBy", createdBy).put("creationSource", creationSource))
        /** `scheduledTasks.setEnabled` · mutation · replay set · scope orchestration:operate · 20000 ms */
        fun setEnabled(id: String, enabled: Boolean): Any? =
            client.call("scheduledTasks.setEnabled", JSONObject().put("id", id).put("enabled", enabled))
        /** `scheduledTasks.delete` · mutation · replay set · scope orchestration:operate · 20000 ms */
        fun delete(id: String): Any? =
            client.call("scheduledTasks.delete", JSONObject().put("id", id))
        /** `scheduledTasks.runNow` · mutation · replay never · scope orchestration:operate · 20000 ms */
        fun runNow(id: String): Any? =
            client.call("scheduledTasks.runNow", JSONObject().put("id", id))
    }

    class Secrets internal constructor(private val client: T3Client) {
        /** `secrets.answerRequest` · mutation · replay never · scope orchestration:operate · 20000 ms */
        fun answerRequest(threadId: String, turnItemId: String, answer: JSONObject): Any? =
            client.call("secrets.answerRequest", JSONObject().put("threadId", threadId).put("turnItemId", turnItemId).put("answer", answer))
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
        /** `server.discoverSourceControl` · read · replay free · scope orchestration:read · 20000 ms */
        fun discoverSourceControl(): Any? =
            client.call("server.discoverSourceControl", JSONObject())
        /** `server.reportClientActivity` · mutation · replay set · scope orchestration:read · 20000 ms */
        fun reportClientActivity(clientId: String, clientKind: String, visible: Boolean, focused: Boolean, recentlyInteracted: Boolean, scopes: JSONArray, observedAt: String, environmentId: String? = null, appState: String? = null, lowPowerMode: String? = null, batteryState: String? = null, networkType: String? = null, ttlMs: Long? = null): Any? =
            client.call("server.reportClientActivity", JSONObject().put("clientId", clientId).put("clientKind", clientKind).put("visible", visible).put("focused", focused).put("recentlyInteracted", recentlyInteracted).put("scopes", scopes).put("observedAt", observedAt).put("environmentId", environmentId).put("appState", appState).put("lowPowerMode", lowPowerMode).put("batteryState", batteryState).put("networkType", networkType).put("ttlMs", ttlMs))
        /** `server.upsertKeybinding` · mutation · replay set · scope settings:write · 20000 ms */
        fun upsertKeybinding(key: String, command: String, `when`: String? = null, replace: JSONObject? = null): Any? =
            client.call("server.upsertKeybinding", JSONObject().put("key", key).put("command", command).put("when", `when`).put("replace", replace))
        /** `server.removeKeybinding` · mutation · replay set · scope settings:write · 20000 ms */
        fun removeKeybinding(key: String, command: String, `when`: String? = null): Any? =
            client.call("server.removeKeybinding", JSONObject().put("key", key).put("command", command).put("when", `when`))
        /** `server.searchAcpRegistry` · read · replay free · scope orchestration:read · 20000 ms */
        fun searchAcpRegistry(query: String, limit: Long? = null): Any? =
            client.call("server.searchAcpRegistry", JSONObject().put("query", query).put("limit", limit))
        /** `server.prepareAcpRegistryAgent` · mutation · replay set · scope providers:manage · 20000 ms */
        fun prepareAcpRegistryAgent(agentId: String): Any? =
            client.call("server.prepareAcpRegistryAgent", JSONObject().put("agentId", agentId))
        /** `server.subscribeServerConfig` · stream · read · replay free · scope orchestration:read */
        fun subscribeServerConfig(environmentThemes: Boolean? = null, usageLimitSources: Boolean? = null, usageLimitsCommand: Boolean? = null): T3Subscription =
            client.subscribeFn("server.subscribeServerConfig", JSONObject().put("environmentThemes", environmentThemes).put("usageLimitSources", usageLimitSources).put("usageLimitsCommand", usageLimitsCommand))
    }

    class Shell internal constructor(private val client: T3Client) {
        /** `shell.openInEditor` · mutation · replay never · scope orchestration:operate · 20000 ms */
        fun openInEditor(cwd: String, editor: String, reveal: Boolean? = null): Any? =
            client.call("shell.openInEditor", JSONObject().put("cwd", cwd).put("editor", editor).put("reveal", reveal))
    }

    class Terminal internal constructor(private val client: T3Client) {
        /** `terminal.open` · mutation · replay set · scope terminal:operate · 20000 ms */
        fun open(threadId: String, terminalId: String, cwd: String, worktreePath: String? = null, cols: Long? = null, rows: Long? = null, env: JSONObject? = null, providerInstanceId: String? = null): Any? =
            client.call("terminal.open", JSONObject().put("threadId", threadId).put("terminalId", terminalId).put("cwd", cwd).put("worktreePath", worktreePath).put("cols", cols).put("rows", rows).put("env", env).put("providerInstanceId", providerInstanceId))
        /** `terminal.write` · mutation · replay never · scope terminal:operate · 20000 ms */
        fun write(threadId: String, terminalId: String, data: String): Any? =
            client.call("terminal.write", JSONObject().put("threadId", threadId).put("terminalId", terminalId).put("data", data))
        /** `terminal.resize` · mutation · replay set · scope terminal:operate · 20000 ms */
        fun resize(threadId: String, terminalId: String, cols: Long, rows: Long): Any? =
            client.call("terminal.resize", JSONObject().put("threadId", threadId).put("terminalId", terminalId).put("cols", cols).put("rows", rows))
        /** `terminal.attach` · stream · mutation · replay set · scope terminal:operate */
        fun attach(threadId: String, terminalId: String, cwd: String? = null, worktreePath: String? = null, cols: Long? = null, rows: Long? = null, env: JSONObject? = null, providerInstanceId: String? = null, restartIfNotRunning: Boolean? = null, resume: Boolean? = null): T3Subscription =
            client.subscribeFn("terminal.attach", JSONObject().put("threadId", threadId).put("terminalId", terminalId).put("cwd", cwd).put("worktreePath", worktreePath).put("cols", cols).put("rows", rows).put("env", env).put("providerInstanceId", providerInstanceId).put("restartIfNotRunning", restartIfNotRunning).put("resume", resume))
    }

    class Vcs internal constructor(private val client: T3Client) {
        /** `vcs.refreshStatus` · read · replay free · scope orchestration:read · 20000 ms */
        fun refreshStatus(cwd: String): Any? =
            client.call("vcs.refreshStatus", JSONObject().put("cwd", cwd))
        /** `vcs.listRefs` · read · replay free · scope orchestration:read · 20000 ms */
        fun listRefs(cwd: String, query: String? = null, cursor: Long? = null, includeMatchingRemoteRefs: Boolean? = null, refKind: String? = null, refresh: Boolean? = null, limit: Long? = null): Any? =
            client.call("vcs.listRefs", JSONObject().put("cwd", cwd).put("query", query).put("cursor", cursor).put("includeMatchingRemoteRefs", includeMatchingRemoteRefs).put("refKind", refKind).put("refresh", refresh).put("limit", limit))
        /** `vcs.createRef` · mutation · replay never · scope source-control:write · 20000 ms */
        fun createRef(cwd: String, refName: String, switchRef: Boolean? = null): Any? =
            client.call("vcs.createRef", JSONObject().put("cwd", cwd).put("refName", refName).put("switchRef", switchRef))
        /** `vcs.switchRef` · mutation · replay set · scope source-control:write · 20000 ms */
        fun switchRef(cwd: String, refName: String): Any? =
            client.call("vcs.switchRef", JSONObject().put("cwd", cwd).put("refName", refName))
        /** `vcs.pull` · mutation · replay set · scope source-control:write · 20000 ms */
        fun pull(cwd: String): Any? =
            client.call("vcs.pull", JSONObject().put("cwd", cwd))
        /** `vcs.init` · mutation · replay set · scope source-control:write · 20000 ms */
        fun init(cwd: String, kind: String? = null): Any? =
            client.call("vcs.init", JSONObject().put("cwd", cwd).put("kind", kind))
        /** `git.preparePullRequestThread` · mutation · replay never · scope one of source-control:write, orchestration:operate · 20000 ms */
        fun preparePullRequestThread(cwd: String, reference: String, mode: String, threadId: String? = null): Any? =
            client.call("git.preparePullRequestThread", JSONObject().put("cwd", cwd).put("reference", reference).put("mode", mode).put("threadId", threadId))
        /** `git.runStackedAction` · stream · mutation · replay never · scope one of source-control:write, orchestration:read */
        fun runStackedAction(actionId: String, cwd: String, action: String, commitMessage: String? = null, featureBranch: Boolean? = null, filePaths: JSONArray? = null, threadId: String? = null, projectId: String? = null): T3Subscription =
            client.subscribeFn("git.runStackedAction", JSONObject().put("actionId", actionId).put("cwd", cwd).put("action", action).put("commitMessage", commitMessage).put("featureBranch", featureBranch).put("filePaths", filePaths).put("threadId", threadId).put("projectId", projectId))
    }

    class WorktreeSetup internal constructor(private val client: T3Client) {
        /** `worktreeSetup.cancel` · mutation · replay set · scope orchestration:operate · 20000 ms */
        fun cancel(threadId: String): Any? =
            client.call("worktreeSetup.cancel", JSONObject().put("threadId", threadId))
        /** `worktreeSetup.subscribeWorktreeSetup` · stream · read · replay free · scope orchestration:read */
        fun subscribeWorktreeSetup(threadId: String): T3Subscription =
            client.subscribeFn("worktreeSetup.subscribeWorktreeSetup", JSONObject().put("threadId", threadId))
    }

    /** Pure functions, offline: no connection is needed or used. */
    class Domain internal constructor(client: T3Client) {
        val approvals = Approvals(client)
        val codeBlocks = CodeBlocks(client)
        val composer = Composer(client)
        val connectionCatalog = ConnectionCatalog(client)
        val history = History(client)
        val i18n = I18n(client)
        val models = Models(client)
        val projection = Projection(client)
        val projects = Projects(client)
        val questions = Questions(client)
        val queue = Queue(client)
        val sidebar = Sidebar(client)

        class Approvals internal constructor(private val client: T3Client) {
            /** `approvals.pending` · array of {requestId, requestKind, createdAt, responseCapability, detail?, appName?, options}, oldest first */
            fun pending(projection: JSONObject): Any? =
                client.domain("approvals.pending", JSONObject().put("projection", projection))
            /** `approvals.command` · command runtime-request.respond, only for a pending live request and an offered decision */
            fun command(thread: String, runtimeRequestId: String, decision: String, projection: JSONObject): Any? =
                client.domain("approvals.command", JSONObject().put("thread", thread).put("runtimeRequestId", runtimeRequestId).put("decision", decision).put("projection", projection))
        }

        class CodeBlocks internal constructor(private val client: T3Client) {
            /** `codeBlocks.parse` · array of {index, language, code, command}: command only for a complete single-line shell fence */
            fun parse(markdown: String): Any? =
                client.domain("codeBlocks.parse", JSONObject().put("markdown", markdown))
        }

        class Composer internal constructor(private val client: T3Client) {
            /** `composer.paths` · array of {kind, label, description, insert, icon} (at most 40) from projects.searchEntries */
            fun paths(result: JSONObject): Any? =
                client.domain("composer.paths", JSONObject().put("result", result))
            /** `composer.catalog` · array of {kind, label, description, insert, icon} (at most 40): skills and slash commands */
            fun catalog(provider: JSONObject, cwd: String, kind: String, query: String? = null): Any? =
                client.domain("composer.catalog", JSONObject().put("provider", provider).put("cwd", cwd).put("kind", kind).put("query", query))
            /** `composer.fileLink` · string: the Markdown file link T3's composer inserts */
            fun fileLink(path: String): Any? =
                client.domain("composer.fileLink", JSONObject().put("path", path))
            /** `composer.attachmentReference` · string: the inline t3-context reference of an uploaded attachment */
            fun attachmentReference(attachment: JSONObject): Any? =
                client.domain("composer.attachmentReference", JSONObject().put("attachment", attachment))
        }

        class ConnectionCatalog internal constructor(private val client: T3Client) {
            /** `connectionCatalog.pairingTarget` · object {httpBaseUrl, pairingCode}: a pairing link, QR payload or host plus code */
            fun pairingTarget(link: String, pairingCode: String? = null): Any? =
                client.domain("connectionCatalog.pairingTarget", JSONObject().put("link", link).put("pairingCode", pairingCode))
        }

        class History internal constructor(private val client: T3Client) {
            /** `history.path` · string: the history page path for request */
            fun path(threadId: String, cursor: String): Any? =
                client.domain("history.path", JSONObject().put("threadId", threadId).put("cursor", cursor))
            /** `history.envelope` · object {projection {thread, messages, turnItems}, historyCursor, hasMoreHistory} */
            fun envelope(page: JSONObject, threadId: String): Any? =
                client.domain("history.envelope", JSONObject().put("page", page).put("threadId", threadId))
        }

        class I18n internal constructor(private val client: T3Client) {
            /** `i18n.translate` · string: the translation, the English source as fallback */
            fun translate(catalog: JSONObject, locale: String, source: String): Any? =
                client.domain("i18n.translate", JSONObject().put("catalog", catalog).put("locale", locale).put("source", source))
            /** `i18n.normalize` · string: a POSIX or BCP 47 locale as lowercase BCP 47 (fr_CA.UTF-8 → fr-ca) */
            fun normalize(locale: String): Any? =
                client.domain("i18n.normalize", JSONObject().put("locale", locale))
        }

        class Models internal constructor(private val client: T3Client) {
            /** `models.catalog` · object {models, providers, favorites, favoritesReady, capabilities, scratchWorkspaceRoot} from server.getConfig */
            fun catalog(config: JSONObject): Any? =
                client.domain("models.catalog", JSONObject().put("config", config))
            /** `models.preferences` · object {favorites, favoritesReady} from server.getSettings */
            fun preferences(settings: JSONObject): Any? =
                client.domain("models.preferences", JSONObject().put("settings", settings))
            /** `models.favoritePayload` · object {patch} for server.updateSettings */
            fun favoritePayload(provider: String, model: String, favorite: Boolean): Any? =
                client.domain("models.favoritePayload", JSONObject().put("provider", provider).put("model", model).put("favorite", favorite))
            /** `models.runtimeModeCommand` · command thread.runtime-mode.set */
            fun runtimeModeCommand(threadId: String, commandId: String, runtimeMode: String): Any? =
                client.domain("models.runtimeModeCommand", JSONObject().put("threadId", threadId).put("commandId", commandId).put("runtimeMode", runtimeMode))
        }

        class Projection internal constructor(private val client: T3Client) {
            /** `projection.apply` · object {projection, changed, applied}: `event` then each of `events` applied in order */
            fun apply(projection: JSONObject, event: JSONObject? = null, events: JSONArray? = null): Any? =
                client.domain("projection.apply", JSONObject().put("projection", projection).put("event", event).put("events", events))
        }

        class Projects internal constructor(private val client: T3Client) {
            /** `projects.identity` · array of {key, monogram, label, kind, icon, colorSlot, colorKey, color, fill}, one per project, slots over all of them */
            fun identity(projects: JSONArray, colors: JSONObject? = null, accent: String? = null): Any? =
                client.domain("projects.identity", JSONObject().put("projects", projects).put("colors", colors).put("accent", accent))
            /** `projects.monogram` · string: the two-letter monogram of a project title */
            fun monogram(name: String): Any? =
                client.domain("projects.monogram", JSONObject().put("name", name))
        }

        class Questions internal constructor(private val client: T3Client) {
            /** `questions.pending` · array of {requestId, createdAt, questions, responseCapability, dismissible}, oldest first */
            fun pending(projection: JSONObject): Any? =
                client.domain("questions.pending", JSONObject().put("projection", projection))
            /** `questions.command` · command runtime-request.respond (answers) or thread.user-input.dismiss */
            fun command(thread: String, runtimeRequestId: String, action: String, answers: JSONObject? = null): Any? =
                client.domain("questions.command", JSONObject().put("thread", thread).put("runtimeRequestId", runtimeRequestId).put("action", action).put("answers", answers))
        }

        class Queue internal constructor(private val client: T3Client) {
            /** `queue.derive` · object {items, hiddenMessageIds, acknowledgedMessageIds, held, activeRunId, canReorder, canSteer} */
            fun derive(projection: JSONObject): Any? =
                client.domain("queue.derive", JSONObject().put("projection", projection))
            /** `queue.command` · command: queued-run.cancel/reorder/edit, queued-message.promote-to-steer or queue.resume (without commandId) */
            fun command(thread: String, action: String, runId: String? = null, beforeRunId: String? = null, targetRunId: String? = null, text: String? = null, attachments: JSONArray? = null, context: JSONObject? = null): Any? =
                client.domain("queue.command", JSONObject().put("thread", thread).put("action", action).put("runId", runId).put("beforeRunId", beforeRunId).put("targetRunId", targetRunId).put("text", text).put("attachments", attachments).put("context", context))
        }

        class Sidebar internal constructor(private val client: T3Client) {
            /** `sidebar.status` · string: approval, input, waiting, working, goal, limited, failed or ready */
            fun status(thread: JSONObject): Any? =
                client.domain("sidebar.status", JSONObject().put("thread", thread))
            /** `sidebar.metadata` · object {sidebarStatus, woke, wokeAt, snoozed, canSnooze, snoozedUntil, hasRuntimeError, runtimeErrorClass, providerHistory, pullRequests, linkedPullRequest, branchPullRequest, pullRequestBadge} */
            fun metadata(thread: JSONObject, now: Long): Any? =
                client.domain("sidebar.metadata", JSONObject().put("thread", thread).put("now", now))
            /** `sidebar.sortTimestamps` · object {inboxAt, activeAnchorAt, lastSentAt, settledSortAt} (ms) */
            fun sortTimestamps(thread: JSONObject): Any? =
                client.domain("sidebar.sortTimestamps", JSONObject().put("thread", thread))
            /** `sidebar.command` · command: thread.settle, thread.unsettle, thread.snooze or thread.unsnooze (without commandId) */
            fun command(action: String, threadId: String, snoozedUntil: String? = null): Any? =
                client.domain("sidebar.command", JSONObject().put("action", action).put("threadId", threadId).put("snoozedUntil", snoozedUntil))
            /** `sidebar.threadAction` · command for the thread actions menu (without commandId) */
            fun threadAction(thread: String, action: String, title: String? = null, enabled: Boolean? = null, confirmed: Boolean? = null, working: Boolean? = null): Any? =
                client.domain("sidebar.threadAction", JSONObject().put("thread", thread).put("action", action).put("title", title).put("enabled", enabled).put("confirmed", confirmed).put("working", working))
            /** `sidebar.snapshot` · object {environments [{id, name, state}], stale, projects, threads, working, pending}: each environment {id, name?, shell?, state?} as ot3's rows, newest first */
            fun snapshot(environments: JSONArray, now: Long): Any? =
                client.domain("sidebar.snapshot", JSONObject().put("environments", environments).put("now", now))
            /** `sidebar.view` · object {sections {active, working, snoozed, settled}, sectionCounts, counts, filterCounts, projects, projectGroups, projectScope, projectIdentities {key: identity}, environments, stale, working, pending, threads, returns}; rows carry detail, state, statusLabel, statusRole and time */
            fun view(environments: JSONArray, now: Long, filter: String? = null, project: String? = null, environmentId: String? = null, query: String? = null, grouped: Boolean? = null, subthreadsExpanded: JSONObject? = null, selectedKey: String? = null, returns: JSONObject? = null, groupingMode: String? = null, groupingOverrides: JSONObject? = null, nativeGrouping: JSONObject? = null, colors: JSONObject? = null, accent: String? = null): Any? =
                client.domain("sidebar.view", JSONObject().put("environments", environments).put("now", now).put("filter", filter).put("project", project).put("environmentId", environmentId).put("query", query).put("grouped", grouped).put("subthreadsExpanded", subthreadsExpanded).put("selectedKey", selectedKey).put("returns", returns).put("groupingMode", groupingMode).put("groupingOverrides", groupingOverrides).put("nativeGrouping", nativeGrouping).put("colors", colors).put("accent", accent))
            /** `sidebar.detail` · object {detail, state, statusLabel, statusRole, time} of one sidebar row */
            fun detail(thread: JSONObject, now: Long? = null): Any? =
                client.domain("sidebar.detail", JSONObject().put("thread", thread).put("now", now))
        }
    }
}
