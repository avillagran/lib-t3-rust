// Generated from bindings/android/describe.json (t3code@3f9448747) by tests/api_describe.rs.
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
    val guestShare = GuestShare(client)
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
        /** `device.easAccount` · read · replay free · scope settings:write · 20000 ms */
        fun easAccount(hostId: String? = null, accessToken: String? = null): Any? =
            client.call("device.easAccount", JSONObject().put("hostId", hostId).put("accessToken", accessToken))
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

    class GuestShare internal constructor(private val client: T3Client) {
        /** `guestShare.create` · mutation · replay never · scope access:write · 20000 ms */
        fun create(threadId: String): Any? =
            client.call("guestShare.create", JSONObject().put("threadId", threadId))
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
        /** `orchestration.getThreadFamily` · read · replay free · scope orchestration:read · 20000 ms */
        fun getThreadFamily(threadId: String): Any? =
            client.call("orchestration.getThreadFamily", JSONObject().put("threadId", threadId))
        /** `orchestration.getWorktreeThreadIds` · read · replay free · scope orchestration:read · 20000 ms */
        fun getWorktreeThreadIds(worktreePath: String): Any? =
            client.call("orchestration.getWorktreeThreadIds", JSONObject().put("worktreePath", worktreePath))
        /** `orchestration.getOlderSettledThreads` · read · replay free · scope orchestration:read · 20000 ms */
        fun getOlderSettledThreads(before: JSONObject, limit: Long, projectIds: JSONArray? = null): Any? =
            client.call("orchestration.getOlderSettledThreads", JSONObject().put("before", before).put("projectIds", projectIds).put("limit", limit))
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
        fun searchThreads(query: String, limit: Long? = null, olderSettledBefore: String? = null, projectIds: JSONArray? = null, environmentLabel: String? = null, titlesOnly: Boolean? = null): Any? =
            client.call("orchestration.searchThreads", JSONObject().put("query", query).put("limit", limit).put("olderSettledBefore", olderSettledBefore).put("projectIds", projectIds).put("environmentLabel", environmentLabel).put("titlesOnly", titlesOnly))
        /** `orchestration.getArchivedShellSnapshot` · read · replay free · scope orchestration:read · 20000 ms */
        fun getArchivedShellSnapshot(): Any? =
            client.call("orchestration.getArchivedShellSnapshot", JSONObject())
        /** `orchestration.subscribeShell` · stream · read · replay free · scope orchestration:read */
        fun subscribeShell(afterSequence: Long? = null, requestCompletionMarker: Boolean? = null, omitIdleSubagents: Boolean? = null, omitOlderSettled: Boolean? = null): T3Subscription =
            client.subscribeFn("orchestration.subscribeShell", JSONObject().put("afterSequence", afterSequence).put("requestCompletionMarker", requestCompletionMarker).put("omitIdleSubagents", omitIdleSubagents).put("omitOlderSettled", omitOlderSettled))
        /** `orchestration.subscribeThread` · stream · read · replay free · scope orchestration:read */
        fun subscribeThread(threadId: String, afterSequence: Long? = null, requestCompletionMarker: Boolean? = null, acceptBoundedSnapshot: Boolean? = null, acceptCompactTurnItems: Boolean? = null, acceptDedupedBodies: Boolean? = null, acceptCompactIds: Boolean? = null): T3Subscription =
            client.subscribeFn("orchestration.subscribeThread", JSONObject().put("threadId", threadId).put("afterSequence", afterSequence).put("requestCompletionMarker", requestCompletionMarker).put("acceptBoundedSnapshot", acceptBoundedSnapshot).put("acceptCompactTurnItems", acceptCompactTurnItems).put("acceptDedupedBodies", acceptDedupedBodies).put("acceptCompactIds", acceptCompactIds))
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
        /** `projects.listIgnoredFiles` · read · replay free · scope filesystem:read · 20000 ms */
        fun listIgnoredFiles(cwd: String): Any? =
            client.call("projects.listIgnoredFiles", JSONObject().put("cwd", cwd))
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
        /** `pullRequests.changeBreakdown` · read · replay free · scope orchestration:read · 20000 ms */
        fun changeBreakdown(projectId: String, repository: String, number: Long, host: String? = null, expectedAccountId: String? = null, allowStale: Boolean? = null): Any? =
            client.call("pullRequests.changeBreakdown", JSONObject().put("projectId", projectId).put("repository", repository).put("number", number).put("host", host).put("expectedAccountId", expectedAccountId).put("allowStale", allowStale))
        /** `pullRequests.filterOptions` · read · replay free · scope orchestration:read · 20000 ms */
        fun filterOptions(kind: String, projectId: String? = null, projectIds: JSONArray? = null, host: String? = null, query: String? = null): Any? =
            client.call("pullRequests.filterOptions", JSONObject().put("kind", kind).put("projectId", projectId).put("projectIds", projectIds).put("host", host).put("query", query))
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
        val activity = Activity(client)
        val approvals = Approvals(client)
        val awareness = Awareness(client)
        val branchPicker = BranchPicker(client)
        val changes = Changes(client)
        val checkpointRevert = CheckpointRevert(client)
        val codeBlocks = CodeBlocks(client)
        val commandOutput = CommandOutput(client)
        val composer = Composer(client)
        val connectionCatalog = ConnectionCatalog(client)
        val diffReview = DiffReview(client)
        val gitControls = GitControls(client)
        val history = History(client)
        val i18n = I18n(client)
        val markdown = Markdown(client)
        val models = Models(client)
        val nativeSettings = NativeSettings(client)
        val planWorkflow = PlanWorkflow(client)
        val projection = Projection(client)
        val projects = Projects(client)
        val providerInstances = ProviderInstances(client)
        val pullRequests = PullRequests(client)
        val questions = Questions(client)
        val queue = Queue(client)
        val quotaSources = QuotaSources(client)
        val reader = Reader(client)
        val scopedSettings = ScopedSettings(client)
        val sidebar = Sidebar(client)
        val subagents = Subagents(client)
        val theme = Theme(client)
        val timeline = Timeline(client)
        val toolInspector = ToolInspector(client)
        val usageSummary = UsageSummary(client)

        class Activity internal constructor(private val client: T3Client) {
            /** `activity.describe` · object {id, type, category, title, command, runId, status, failed, active} of a turn item (or its projected row), or null for messages, checkpoints, plans and errors; never the tool's input or output */
            fun describe(row: JSONObject): Any? =
                client.domain("activity.describe", JSONObject().put("row", row))
        }

        class Approvals internal constructor(private val client: T3Client) {
            /** `approvals.pending` · array of {requestId, requestKind, createdAt, responseCapability, detail?, appName?, options}, oldest first */
            fun pending(projection: JSONObject): Any? =
                client.domain("approvals.pending", JSONObject().put("projection", projection))
            /** `approvals.command` · command runtime-request.respond, only for a pending live request and an offered decision */
            fun command(thread: String, runtimeRequestId: String, decision: String, projection: JSONObject): Any? =
                client.domain("approvals.command", JSONObject().put("thread", thread).put("runtimeRequestId", runtimeRequestId).put("decision", decision).put("projection", projection))
        }

        class Awareness internal constructor(private val client: T3Client) {
            /** `awareness.view` · object {counts, rows [{route {environmentId, threadId}, phase, title, project, host, status, since?, expiresAt}], card {active, phase, title, subtitle, status, chip? {kind working, live, approve or answer, count}, since?, expiresAt, route, rows (at most 5)} or null, alerts [{id, group attention or finished, phase, title, body, status, route, expiresAt?}], summaries [{group, title, body, lines, route}], baseline, nextDeadline}: T3's phases, card and lifetimes and ot3's alert key over sidebar.view's environments; pass baseline back on the next call and call again at nextDeadline (ms); texts in locale */
            fun view(environments: JSONArray, now: Long, baseline: JSONObject? = null, onScreen: JSONObject? = null, preferences: JSONObject? = null, lifecycle: String? = null, locale: String? = null): Any? =
                client.domain("awareness.view", JSONObject().put("environments", environments).put("now", now).put("baseline", baseline).put("onScreen", onScreen).put("preferences", preferences).put("lifecycle", lifecycle).put("locale", locale))
        }

        class BranchPicker internal constructor(private val client: T3Client) {
            /** `branchPicker.dedupe` · array: vcs.listRefs refs without the origin refs whose local branch is listed */
            fun dedupe(refs: JSONArray): Any? =
                client.domain("branchPicker.dedupe", JSONObject().put("refs", refs))
            /** `branchPicker.deriveLocal` · string: the local branch name of a remote ref (origin/feature/x → feature/x) */
            fun deriveLocal(name: String): Any? =
                client.domain("branchPicker.deriveLocal", JSONObject().put("name", name))
            /** `branchPicker.sanitize` · string: a new branch name with whitespace runs as single dashes */
            fun sanitize(name: String): Any? =
                client.domain("branchPicker.sanitize", JSONObject().put("name", name))
            /** `branchPicker.selectionTarget` · object {checkoutCwd, nextWorktreePath, reuseExistingWorktree}: where to switch to the picked ref and the thread's next worktree */
            fun selectionTarget(projectCwd: String, ref: JSONObject, worktreePath: String? = null): Any? =
                client.domain("branchPicker.selectionTarget", JSONObject().put("projectCwd", projectCwd).put("worktreePath", worktreePath).put("ref", ref))
            /** `branchPicker.mismatch` · object {threadBranch, currentBranch, canRestore} when the checkout is not on the thread's branch, else null */
            fun mismatch(worktreePath: String? = null, threadBranch: String? = null, currentBranch: String? = null): Any? =
                client.domain("branchPicker.mismatch", JSONObject().put("worktreePath", worktreePath).put("threadBranch", threadBranch).put("currentBranch", currentBranch))
            /** `branchPicker.temporaryBranch` · bool: a temporary worktree branch (t3-/t3/ or t3code-/t3code/ plus 8 hex, or t3code/ plus a UUID v4) */
            fun temporaryBranch(name: String): Any? =
                client.domain("branchPicker.temporaryBranch", JSONObject().put("name", name))
        }

        class Changes internal constructor(private val client: T3Client) {
            /** `changes.summaries` · array of {id, runId, capturedAt, fromTurnCount, toTurnCount, files [{path, kind, additions, deletions}], additions, deletions}, one per run with ready checkpoint files */
            fun summaries(projection: JSONObject): Any? =
                client.domain("changes.summaries", JSONObject().put("projection", projection))
        }

        class CheckpointRevert internal constructor(private val client: T3Client) {
            /** `checkpointRevert.canPreview` · bool: offer "Edit from here" on this user message (a ready checkpoint for its run and a turn-start intent, when known) */
            fun canPreview(projection: JSONObject, message: JSONObject): Any? =
                client.domain("checkpointRevert.canPreview", JSONObject().put("projection", projection).put("message", message))
            /** `checkpointRevert.reference` · object {threadId, messageId, messageRunId, messageCheckpointId, checkpointId, checkpointRunId, scopeId, providerThreadId, providerInstanceId, turnCount, priorRollbackRequestId}: the checkpoint just before the message, on the raw projection (revert_* when unknown or stale) */
            fun reference(projection: JSONObject, messageId: String): Any? =
                client.domain("checkpointRevert.reference", JSONObject().put("projection", projection).put("messageId", messageId))
            /** `checkpointRevert.safety` · object {safety, allowed, error}: the live gate {canOperate, connected, supportsConversationRollback, working, sendBusy, connecting, pendingAttachments, rewinding} (anything but true is false) and the first reason a rewind is refused, or null */
            fun safety(safety: JSONObject): Any? =
                client.domain("checkpointRevert.safety", JSONObject().put("safety", safety))
            /** `checkpointRevert.command` · command checkpoint.rollback {commandId omarchy-<requestId>, threadId, scopeId, checkpointId, restoreFiles} for first admission: input {confirmed, restoreFiles, requestId (UUID), environment, thread, reference} with the reference still fresh and the safety gate open */
            fun command(input: JSONObject, projection: JSONObject, safety: JSONObject): Any? =
                client.domain("checkpointRevert.command", JSONObject().put("input", input).put("projection", projection).put("safety", safety))
            /** `checkpointRevert.confirmedCommand` · command checkpoint.rollback from the input alone, to compare with an already persisted outbox command on an explicit retry */
            fun confirmedCommand(input: JSONObject): Any? =
                client.domain("checkpointRevert.confirmedCommand", JSONObject().put("input", input))
            /** `checkpointRevert.observation` · object {state: pending, complete, failed or superseded, error?}: the rewind as the thread's read-back shows it */
            fun observation(input: JSONObject, projection: JSONObject): Any? =
                client.domain("checkpointRevert.observation", JSONObject().put("input", input).put("projection", projection))
            /** `checkpointRevert.recoveryMessage` · object {id, runId, text, attachments, context}: the message to restore to the composer, captured before dispatch */
            fun recoveryMessage(projection: JSONObject, reference: JSONObject): Any? =
                client.domain("checkpointRevert.recoveryMessage", JSONObject().put("projection", projection).put("reference", reference))
            /** `checkpointRevert.appendPrompt` · string: the composer draft with the restored prompt after it (a blank line between) */
            fun appendPrompt(current: String, restored: String): Any? =
                client.domain("checkpointRevert.appendPrompt", JSONObject().put("current", current).put("restored", restored))
        }

        class CodeBlocks internal constructor(private val client: T3Client) {
            /** `codeBlocks.parse` · array of {index, language, code, command}: command only for a complete single-line shell fence */
            fun parse(markdown: String): Any? =
                client.domain("codeBlocks.parse", JSONObject().put("markdown", markdown))
        }

        class CommandOutput internal constructor(private val client: T3Client) {
            /** `commandOutput.apply` · object {state,text,running,truncated,error}: incremental ANSI/VT stripping, CR overwrite, UTF-8 safe 64KiB tail; reference changes reset */
            fun apply(reference: JSONObject, frames: JSONArray, state: JSONObject? = null, closed: Boolean? = null, error: String? = null): Any? =
                client.domain("commandOutput.apply", JSONObject().put("state", state).put("reference", reference).put("frames", frames).put("closed", closed).put("error", error))
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
            /** `composer.newThread` · object {projects,project,modelSelection,runtimeMode,workspaceStrategy,missing,ready,request?}: source/project/sibling/catalog/draft defaults and validated durable launch recipe; automatic host route requires host support */
            fun newThread(input: JSONObject): Any? =
                client.domain("composer.newThread", JSONObject().put("input", input))
            /** `composer.transform` · object: completion/replace/list/recall transform; offsets are UTF-16, stale expectedText rejected; host owns IME and undo */
            fun transform(input: JSONObject): Any? =
                client.domain("composer.transform", JSONObject().put("input", input))
        }

        class ConnectionCatalog internal constructor(private val client: T3Client) {
            /** `connectionCatalog.pairingTarget` · object {httpBaseUrl, pairingCode}: a pairing link, QR payload or host plus code */
            fun pairingTarget(link: String, pairingCode: String? = null): Any? =
                client.domain("connectionCatalog.pairingTarget", JSONObject().put("link", link).put("pairingCode", pairingCode))
        }

        class DiffReview internal constructor(private val client: T3Client) {
            /** `diffReview.present` · object {files [{path, oldPath, kind, binary, missing?, additions, deletions, summaryAdditions?, summaryDeletions?, rows [{kind: hunk, addition, deletion, context or meta, text, oldLine, newLine}], splitRows [{kind, left, right}]}], additions, deletions, fromTurnCount, toTurnCount} from a diff RPC result {diff} */
            fun present(result: JSONObject, summary: JSONObject? = null): Any? =
                client.domain("diffReview.present", JSONObject().put("result", result).put("summary", summary))
            /** `diffReview.scopedResult` · diffReview.present's object for a receipt of the still-open request, null for a late or other request; the receipt's error otherwise */
            fun scopedResult(expected: JSONObject, receipt: JSONObject, summary: JSONObject): Any? =
                client.domain("diffReview.scopedResult", JSONObject().put("expected", expected).put("receipt", receipt).put("summary", summary))
        }

        class GitControls internal constructor(private val client: T3Client) {
            /** `gitControls.scope` · object {environment, thread, requestId, projectId, cwd, projectRoot, worktreePath}: the thread's checkout resolved against the shell snapshot (project threads only; git_scope_changed when projectId or cwd moved) */
            fun scope(shell: JSONObject, environment: String, thread: String, requestId: String, projectId: String, cwd: String): Any? =
                client.domain("gitControls.scope", JSONObject().put("shell", shell).put("environment", environment).put("thread", thread).put("requestId", requestId).put("projectId", projectId).put("cwd", cwd))
            /** `gitControls.resolveScripts` · array of the project's scripts {id, name, command, icon, runOnWorktreeCreate, …} with T3's precedence (project override, defaults, legacy override, project scripts) */
            fun resolveScripts(settings: JSONObject, project: JSONObject): Any? =
                client.domain("gitControls.resolveScripts", JSONObject().put("settings", settings).put("project", project))
            /** `gitControls.actions` · object {commit, push, pull, create_pr, commit_push, commit_push_pr} of bools: what the Git menu offers for a vcs.refreshStatus result */
            fun actions(status: JSONObject): Any? =
                client.domain("gitControls.actions", JSONObject().put("status", status))
            /** `gitControls.present` · object {scope, status, scripts, actions, revision}: the Git & changes sheet; revision fingerprints checkout, status and scripts (not the requestId) */
            fun present(scope: JSONObject, status: JSONObject, scripts: JSONArray): Any? =
                client.domain("gitControls.present", JSONObject().put("scope", scope).put("status", status).put("scripts", scripts))
            /** `gitControls.scopedResult` · the receipt's result for the sheet's still-open scope and request, null for a late or other checkout; the receipt's error otherwise */
            fun scopedResult(expected: JSONObject, receipt: JSONObject): Any? =
                client.domain("gitControls.scopedResult", JSONObject().put("expected", expected).put("receipt", receipt))
            /** `gitControls.buildAction` · object {scope, revision, actionId, operation, steps [{tag, payload}], retryAutomatically: false}: a confirmed action (intent {scope, revision, confirmed, actionId (UUID), operation, defaultBranchConfirmed?, commitMessage?, filePaths?, scriptId?}) as git.runStackedAction, vcs.pull or terminal.open + terminal.write steps */
            fun buildAction(current: JSONObject, intent: JSONObject): Any? =
                client.domain("gitControls.buildAction", JSONObject().put("current", current).put("intent", intent))
            /** `gitControls.buildVerifiedAction` · gitControls.buildAction's plan, only when a fresh read right before dispatch has the same revision (git_stale_result otherwise) */
            fun buildVerifiedAction(current: JSONObject, fresh: JSONObject, intent: JSONObject): Any? =
                client.domain("gitControls.buildVerifiedAction", JSONObject().put("current", current).put("fresh", fresh).put("intent", intent))
        }

        class History internal constructor(private val client: T3Client) {
            /** `history.path` · string: the history page path for request */
            fun path(threadId: String, cursor: String): Any? =
                client.domain("history.path", JSONObject().put("threadId", threadId).put("cursor", cursor))
            /** `history.envelope` · object {projection {thread, messages, turnItems}, historyCursor, hasMoreHistory} */
            fun envelope(page: JSONObject, threadId: String): Any? =
                client.domain("history.envelope", JSONObject().put("page", page).put("threadId", threadId))
            /** `history.mergeView` · object {older,timeline,cursor,hasMoreHistory,applied}: presentation only; reject stale owner/cursor, recent wins */
            fun mergeView(older: JSONArray, recent: JSONArray, owner: String, page: JSONArray? = null, pageOwner: String? = null, cursor: String? = null, pageCursor: String? = null, nextCursor: String? = null, hasMoreHistory: Boolean? = null): Any? =
                client.domain("history.mergeView", JSONObject().put("older", older).put("recent", recent).put("page", page).put("owner", owner).put("pageOwner", pageOwner).put("cursor", cursor).put("pageCursor", pageCursor).put("nextCursor", nextCursor).put("hasMoreHistory", hasMoreHistory))
        }

        class I18n internal constructor(private val client: T3Client) {
            /** `i18n.translate` · string: the translation, the English source as fallback */
            fun translate(catalog: JSONObject, locale: String, source: String): Any? =
                client.domain("i18n.translate", JSONObject().put("catalog", catalog).put("locale", locale).put("source", source))
            /** `i18n.normalize` · string: a POSIX or BCP 47 locale as lowercase BCP 47 (fr_CA.UTF-8 → fr-ca) */
            fun normalize(locale: String): Any? =
                client.domain("i18n.normalize", JSONObject().put("locale", locale))
            /** `i18n.t` · string: ot3's translation of an English source string for a locale list (fr_CA.UTF-8:en), else the language's, English's or the source; {name} placeholders from args, Arabic isolated with U+2068/U+2069 */
            fun t(locale: String, key: String, args: JSONObject? = null): Any? =
                client.domain("i18n.t", JSONObject().put("locale", locale).put("key", key).put("args", args))
            /** `i18n.catalog` · object {locale, language, rtl, strings {source: translation}}: every string of the catalog for the locale as i18n.t gives it without args (2027 sources: Desktop additions and retained compatible sources; 20 languages) */
            fun catalog(locale: String): Any? =
                client.domain("i18n.catalog", JSONObject().put("locale", locale))
            /** `i18n.resolve` · object {locale, language, rtl}: the first locale of the list the catalog has (by itself or its language), else the first */
            fun resolve(locale: String): Any? =
                client.domain("i18n.resolve", JSONObject().put("locale", locale))
            /** `i18n.languages` · array of {id, name}: ot3's language picker, system first (its name translated for locale), then each catalog language in its own name */
            fun languages(locale: String? = null): Any? =
                client.domain("i18n.languages", JSONObject().put("locale", locale))
        }

        class Markdown internal constructor(private val client: T3Client) {
            /** `markdown.blocks` · array of blocks {type: paragraph, heading, list, quote, code, table, image, math or rule} with flat inline spans (text with strong/emphasis/strikethrough/link, code, break, image, math); links classified github, web, thread, file or anchor; unsafe links dropped */
            fun blocks(markdown: String, mathRenderingEnabled: Boolean? = null): Any? =
                client.domain("markdown.blocks", JSONObject().put("markdown", markdown).put("mathRenderingEnabled", mathRenderingEnabled))
            /** `markdown.link` · object {href, kind: github, web, thread, file or anchor, fileType?} or null when ot3 would not follow the link */
            fun link(href: String): Any? =
                client.domain("markdown.link", JSONObject().put("href", href))
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
            /** `models.permissionView` · object {current, rows}: advertised runtime modes; absent/empty advertisement uses all definitions */
            fun permissionView(catalog: JSONObject, selection: JSONObject? = null, runtimeMode: String? = null): Any? =
                client.domain("models.permissionView", JSONObject().put("catalog", catalog).put("selection", selection).put("runtimeMode", runtimeMode))
        }

        class NativeSettings internal constructor(private val client: T3Client) {
            /** `nativeSettings.snapshot` · object {fields [{key, label, section, kind: bool, choice, days, text or model, value, resetValue?, choices?, description?, …}]}: server.getSettings as the settings pages' fields; never credentials or provider configuration */
            fun snapshot(settings: JSONObject): Any? =
                client.domain("nativeSettings.snapshot", JSONObject().put("settings", settings))
            /** `nativeSettings.patch` · object {patch} for server.updateSettings from {key, value} (value null clears a nullable model); with server.getConfig as config, models and storage settings are checked against the server */
            fun patch(input: JSONObject, config: JSONObject? = null): Any? =
                client.domain("nativeSettings.patch", JSONObject().put("input", input).put("config", config))
            /** `nativeSettings.description` · string: a field's English description (empty when it has none) */
            fun description(key: String): Any? =
                client.domain("nativeSettings.description", JSONObject().put("key", key))
            /** `nativeSettings.modelSelection` · object {instanceId, model, options? [{id, value}]}: a checked model selection */
            fun modelSelection(selection: JSONObject): Any? =
                client.domain("nativeSettings.modelSelection", JSONObject().put("selection", selection))
            /** `nativeSettings.idleCompaction` · string: the idle compaction mode (onReturn, beforeCacheExpires or off), the provider instance's override first when instanceId is given; null when the server has none */
            fun idleCompaction(settings: JSONObject, instanceId: String? = null): Any? =
                client.domain("nativeSettings.idleCompaction", JSONObject().put("settings", settings).put("instanceId", instanceId))
            /** `nativeSettings.idleCompactionPatch` · object {patch}: an idleCompaction patch in the form this server reads (compactIdleThreads on older servers) */
            fun idleCompactionPatch(settings: JSONObject, patch: JSONObject): Any? =
                client.domain("nativeSettings.idleCompactionPatch", JSONObject().put("settings", settings).put("patch", patch))
            /** `nativeSettings.threadDefaults` · object {defaultModelSelection, modelDefaultConfigured, defaultRuntimeMode}: a new thread's defaults, the project's over the environment's */
            fun threadDefaults(settings: JSONObject, projectId: String): Any? =
                client.domain("nativeSettings.threadDefaults", JSONObject().put("settings", settings).put("projectId", projectId))
        }

        class PlanWorkflow internal constructor(private val client: T3Client) {
            /** `planWorkflow.view` · object {plans [{id, threadId, runId, status, markdown, title, displayMarkdown, blocks, updatedAt}], ready (the plan the banner offers to implement, or null), error?}: the projection's proposed plans with markdown.blocks of their display Markdown */
            fun view(projection: JSONObject, mathRenderingEnabled: Boolean? = null): Any? =
                client.domain("planWorkflow.view", JSONObject().put("projection", projection).put("mathRenderingEnabled", mathRenderingEnabled))
            /** `planWorkflow.followUp` · object {text, interactionMode, sourcePlanRef?}: an empty draft implements the plan (interactionMode default, sourcePlanRef {threadId, planId}); a non-empty one is feedback in plan mode */
            fun followUp(plan: JSONObject, threadId: String, draft: String? = null): Any? =
                client.domain("planWorkflow.followUp", JSONObject().put("plan", plan).put("threadId", threadId).put("draft", draft))
            /** `planWorkflow.attachSourceRef` · command: the message.dispatch command with the follow-up's sourcePlanRef, also when it targets a new thread */
            fun attachSourceRef(command: JSONObject, followUp: JSONObject): Any? =
                client.domain("planWorkflow.attachSourceRef", JSONObject().put("command", command).put("followUp", followUp))
            /** `planWorkflow.implementationTitle` · string: the new thread's title for "Implement in new thread" (Implement <plan title>, else Implement plan; at most 255 characters) */
            fun implementationTitle(markdown: String): Any? =
                client.domain("planWorkflow.implementationTitle", JSONObject().put("markdown", markdown))
            /** `planWorkflow.mode` · object {enabled, interactionMode}: Plan / Build as the composer offers it for the thread's provider */
            fun mode(planModeEnabled: Boolean, requested: String, provider: JSONObject? = null): Any? =
                client.domain("planWorkflow.mode", JSONObject().put("planModeEnabled", planModeEnabled).put("provider", provider).put("requested", requested))
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
            /** `projects.mutation` · validated project mutation including durable caller commandId for projects.mutate */
            fun mutation(input: JSONObject, commandId: String): Any? =
                client.domain("projects.mutation", JSONObject().put("input", input).put("commandId", commandId))
        }

        class ProviderInstances internal constructor(private val client: T3Client) {
            /** `providerInstances.projection` · safe provider instance fields and secret presence flags; never credentials */
            fun projection(settings: JSONObject, config: JSONObject, writable: Boolean): Any? =
                client.domain("providerInstances.projection", JSONObject().put("settings", settings).put("config", config).put("writable", writable))
            /** `providerInstances.mutation` · object {patch,providerInstanceMutation}: validated atomic server.updateSettings arguments */
            fun mutation(input: JSONObject, settings: JSONObject): Any? =
                client.domain("providerInstances.mutation", JSONObject().put("input", input).put("settings", settings))
            /** `providerInstances.confirmTarget` · bool: mutation persisted in authoritative settings readback */
            fun confirmTarget(action: String, instanceId: String, payload: JSONObject, saved: JSONObject): Any? =
                client.domain("providerInstances.confirmTarget", JSONObject().put("action", action).put("instanceId", instanceId).put("payload", payload).put("saved", saved))
        }

        class PullRequests internal constructor(private val client: T3Client) {
            /** `pullRequests.sidebarBadge` · object {kind: single, multiple or stack, text, state: open, draft, merged, closed or unknown, current, links} of a thread's linked pull requests, or null */
            fun sidebarBadge(thread: JSONObject): Any? =
                client.domain("pullRequests.sidebarBadge", JSONObject().put("thread", thread))
            /** `pullRequests.request` · object {method, payload}: the typed pullRequests.list, detail, checks or activity call and its args (list: 50 per page, state open and involvement all by default), or method diff with the /api/pull-requests/diff query; reference is {projectId, repository, host, number} */
            fun request(action: String, state: String? = null, involvement: String? = null, query: String? = null, cursors: JSONObject? = null, reference: JSONObject? = null, cursor: String? = null): Any? =
                client.domain("pullRequests.request", JSONObject().put("action", action).put("state", state).put("involvement", involvement).put("query", query).put("cursors", cursors).put("reference", reference).put("cursor", cursor))
            /** `pullRequests.payload` · object {method, payload}: the typed pullRequests.comment, submitReview, runAction, reviewerCandidates, labelCandidates, requestReviewers or setLabels call and its args, after the page's rules (a comment needs words; only the page's actions) */
            fun payload(kind: String, reference: JSONObject, body: String? = null, verdict: String? = null, action: String? = null, mergeMethod: String? = null, reviewers: JSONArray? = null, requested: Boolean? = null, labels: JSONArray? = null, applied: Boolean? = null): Any? =
                client.domain("pullRequests.payload", JSONObject().put("kind", kind).put("reference", reference).put("body", body).put("verdict", verdict).put("action", action).put("mergeMethod", mergeMethod).put("reviewers", reviewers).put("requested", requested).put("labels", labels).put("applied", applied))
            /** `pullRequests.threadCommand` · command thread.pull-request.link, unlink or watch for a reference {host, repository, number, url}; commandId only when given */
            fun threadCommand(thread: String, link: String, reference: JSONObject, commandId: String? = null): Any? =
                client.domain("pullRequests.threadCommand", JSONObject().put("thread", thread).put("link", link).put("reference", reference).put("commandId", commandId))
            /** `pullRequests.listView` · object {items}: later pages replace matching environment/host/repository/number, newest updatedAt first */
            fun listView(pages: JSONArray): Any? =
                client.domain("pullRequests.listView", JSONObject().put("pages", pages))
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

        class QuotaSources internal constructor(private val client: T3Client) {
            /** `quotaSources.limits` · object {checkedAt, windows [{id, label, kind: session, weekly, monthly or other, usedPercent, resetsAt?, windowDurationMins?}], credentialFingerprint?, unavailable? {reason}, resetCredits? {availableCount, nextExpiresAt?}, externalUsage? {label, url}}, or null when unreadable */
            fun limits(usageLimits: JSONObject): Any? =
                client.domain("quotaSources.limits", JSONObject().put("usageLimits", usageLimits))
            /** `quotaSources.providers` · array of {instanceId, driver, enabled, installed, availability, auth {status, email?, workspaceId?, label?}, displayName?, accentColor?, usageLimits?}: public provider statuses, never their configuration */
            fun providers(providers: JSONArray): Any? =
                client.domain("quotaSources.providers", JSONObject().put("providers", providers))
            /** `quotaSources.sources` · array of {id, label, accounts [{id, driver, usageLimits, email?, workspaceId?, plan?}], error?}: usage-limit sources with readable accounts */
            fun sources(sources: JSONArray): Any? =
                client.domain("quotaSources.sources", JSONObject().put("sources", sources))
            /** `quotaSources.apply` · object {state {providers, usageLimitSources, connected}, changed}: one server.subscribeServerConfig event (version 1) applied to the quota strip's state */
            fun apply(event: JSONObject, state: JSONObject? = null): Any? =
                client.domain("quotaSources.apply", JSONObject().put("state", state).put("event", event))
            /** `quotaSources.present` · object {providers,notices,external}: workspace-aware fresh account pooling, pace, missing segments and reset translation keys */
            fun present(environments: JSONArray, now: Long): Any? =
                client.domain("quotaSources.present", JSONObject().put("environments", environments).put("now", now))
        }

        class Reader internal constructor(private val client: T3Client) {
            /** `reader.threadReference` · string: canonical t3-thread://v1 reference with percent-encoded id */
            fun threadReference(threadId: String): Any? =
                client.domain("reader.threadReference", JSONObject().put("threadId", threadId))
            /** `reader.reference` · tagged external/thread/document/anchor/unsupported destination; remote paths never expand local HOME */
            fun reference(href: String, environmentId: String, threadId: String, environments: JSONArray): Any? =
                client.domain("reader.reference", JSONObject().put("href", href).put("environmentId", environmentId).put("threadId", threadId).put("environments", environments))
        }

        class ScopedSettings internal constructor(private val client: T3Client) {
            /** `scopedSettings.effective` · object {settings, sources {key: environment, project or t3.json}, overrides}: a project's effective settings over the environment's and its t3.json */
            fun effective(settings: JSONObject, project: JSONObject? = null, file: JSONObject? = null): Any? =
                client.domain("scopedSettings.effective", JSONObject().put("settings", settings).put("project", project).put("file", file))
            /** `scopedSettings.projectPatch` · object {patch {projectSettingsOverrides {projectId: entry}}}: a project override that keeps its other keys (setting_environment_only for environment keys) */
            fun projectPatch(settings: JSONObject, project: JSONObject, key: String, patch: JSONObject, clear: Boolean? = null): Any? =
                client.domain("scopedSettings.projectPatch", JSONObject().put("settings", settings).put("project", project).put("key", key).put("patch", patch).put("clear", clear))
            /** `scopedSettings.projectFile` · object: a project's t3.json (comments and trailing commas allowed), or null when it is missing or invalid */
            fun projectFile(contents: String): Any? =
                client.domain("scopedSettings.projectFile", JSONObject().put("contents", contents))
            /** `scopedSettings.validatedTargets` · array of {environment, projectId}: the scope's targets, 1 to 64, all project-scoped or all not (settings_scope_unavailable otherwise) */
            fun validatedTargets(scope: JSONObject): Any? =
                client.domain("scopedSettings.validatedTargets", JSONObject().put("scope", scope))
            /** `scopedSettings.writePatch` · object {patch} for one target's server.updateSettings from {key, action: update or clear, value}: project overrides for a project, environment patches otherwise */
            fun writePatch(input: JSONObject, config: JSONObject, settings: JSONObject, writeAllowed: Boolean, project: JSONObject? = null): Any? =
                client.domain("scopedSettings.writePatch", JSONObject().put("input", input).put("config", config).put("settings", settings).put("project", project).put("writeAllowed", writeAllowed))
            /** `scopedSettings.writeWasDropped` · bool: the server accepted the patch but the read-back still has the old value */
            fun writeWasDropped(next: JSONObject, before: JSONObject, saved: JSONObject): Any? =
                client.domain("scopedSettings.writeWasDropped", JSONObject().put("next", next).put("before", before).put("saved", saved))
            /** `scopedSettings.targetView` · object {target, fields (each with source, scopeWritable, overridden, projectScoped), writeAllowed, storageCapabilities, catalog}: one target's page */
            fun targetView(target: JSONObject, config: JSONObject, settings: JSONObject, writeAllowed: Boolean, project: JSONObject? = null, file: JSONObject? = null): Any? =
                client.domain("scopedSettings.targetView", JSONObject().put("target", target).put("config", config).put("settings", settings).put("project", project).put("file", file).put("writeAllowed", writeAllowed))
            /** `scopedSettings.aggregate` · object {fields}: the fields every target has, with mixed, source (mixed when they differ), overridden and scopeWritable over all of them */
            fun aggregate(views: JSONArray): Any? =
                client.domain("scopedSettings.aggregate", JSONObject().put("views", views))
            /** `scopedSettings.page` · object {fields, storageScopes, scope, modelCatalog, targets, errors, savedTargets, complete, writable}: the settings page over every target */
            fun page(scope: JSONObject, views: JSONArray, requested: Long, errors: JSONArray? = null, saved: JSONArray? = null): Any? =
                client.domain("scopedSettings.page", JSONObject().put("scope", scope).put("views", views).put("errors", errors).put("saved", saved).put("requested", requested))
            /** `scopedSettings.projectCatalogConfig` · object: server.getConfig with the project's provider enable overrides applied (for models.catalog) */
            fun projectCatalogConfig(config: JSONObject, settings: JSONObject, project: JSONObject? = null): Any? =
                client.domain("scopedSettings.projectCatalogConfig", JSONObject().put("config", config).put("settings", settings).put("project", project))
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
            /** `sidebar.pinCommands` · array of validated thread.pin.reorder commands using caller-persisted commandIds keyed by thread id */
            fun pinCommands(plan: JSONArray, commandIds: JSONObject): Any? =
                client.domain("sidebar.pinCommands", JSONObject().put("plan", plan).put("commandIds", commandIds))
            /** `sidebar.pinPlan` · array of {id,orderKey}: fractional move or deterministic rebalance; persist caller commandIds before dispatch */
            fun pinPlan(orderedIds: JSONArray, keys: JSONObject, movedId: String): Any? =
                client.domain("sidebar.pinPlan", JSONObject().put("orderedIds", orderedIds).put("keys", keys).put("movedId", movedId))
        }

        class Subagents internal constructor(private val client: T3Client) {
            /** `subagents.buckets` · object {working, done, failed, stopped, idle}: status counts (queued, running and waiting are working) */
            fun buckets(subagents: JSONArray): Any? =
                client.domain("subagents.buckets", JSONObject().put("subagents", subagents))
            /** `subagents.describe` · object {live, settled, label, tone, detail, elapsed}: one subagent row; label is an English source string (Queued, Running, Waiting, Idle · resumable, Completed, Failed, Stopped or Unknown), tone the colour's state (working, waiting, done, failed or idle), elapsed like 4m 09s */
            fun describe(subagent: JSONObject, now: Long): Any? =
                client.domain("subagents.describe", JSONObject().put("subagent", subagent).put("now", now))
            /** `subagents.cards` · object {subagents [{subagent, view}], group, anyLive, expanded, counts, header {source, args}, summary [{source, args}]}: a timeline subagents row's cards overlaid with threadState.subagents; header and summary parts are catalog sources ({n} subagents; {n} working, done, failed, stopped, idle) joined with " · " */
            fun cards(cards: JSONArray, now: Long, threadSubagents: JSONArray? = null): Any? =
                client.domain("subagents.cards", JSONObject().put("cards", cards).put("threadSubagents", threadSubagents).put("now", now))
            /** `subagents.fromThread` · object {id thread:<key>, title, status, childThreadId, startedAt, updatedAt, providerInstanceId, model, fromThread: true}: a sidebar sub-thread seen as a subagent */
            fun fromThread(thread: JSONObject): Any? =
                client.domain("subagents.fromThread", JSONObject().put("thread", thread))
        }

        class Theme internal constructor(private val client: T3Client) {
            /** `theme.tokens` · object {id, name, appearance (light when the background's Rec.709 luminance > 0.5), luminance, mode, base, roles {dim, statusMuted, divider, surface, card, cardBorder, changesCard, rowSelected, rowPressed, rowDraft, field, fieldBorder, switchOff, inlineCode, composerBorder, composerFocus, working, done, waiting, failed, warning, input, onAccent}, layers {role: {color, alpha}}, statusColors {detail: colour}, projectPalette, projectFills, sky {lightSky, star, meteor, ink, header {brightness, nebula, meteorRate, base}, panel}, document {appearance, variables}, mcpTheme}: ot3's colours for an Omarchy colors.toml, each role the opaque #rrggbb ot3 draws */
            fun tokens(colors: String, id: String? = null): Any? =
                client.domain("theme.tokens", JSONObject().put("colors", colors).put("id", id))
        }

        class Timeline internal constructor(private val client: T3Client) {
            /** `timeline.view` · object {timeline [rows with role user, assistant, activity, changes, context, plan, secret, subagents or error, each with rev; assistant rows and plans carry markdown.blocks], threadState, activity, approvals, questions, tasks, queue, planView, historyCursor, hasMoreHistory, payloadBudgetExceeded} */
            fun view(projection: JSONObject, historyCursor: String? = null, hasMoreHistory: Boolean? = null, payloadBudgetExceeded: Boolean? = null, mathRenderingEnabled: Boolean? = null): Any? =
                client.domain("timeline.view", JSONObject().put("projection", projection).put("historyCursor", historyCursor).put("hasMoreHistory", hasMoreHistory).put("payloadBudgetExceeded", payloadBudgetExceeded).put("mathRenderingEnabled", mathRenderingEnabled))
            /** `timeline.liveActivity` · array of the active run's in-flight activity {id, type, category, title, command, runId, status, failed, active, inspector}, newest first (at most 3) */
            fun liveActivity(projection: JSONObject): Any? =
                client.domain("timeline.liveActivity", JSONObject().put("projection", projection))
            /** `timeline.minimap` · object {items [{id, rowIndex, userText, assistantText, userPreview, assistantPreview}], visible (two items or more), currentIndex, visibleIds}: one marker per user row of timeline.view's timeline, paired with the last assistant reply before the next user row; currentIndex and visibleIds follow the rows on screen */
            fun minimap(rows: JSONArray, visibleRowIds: JSONArray? = null, atEnd: Boolean? = null): Any? =
                client.domain("timeline.minimap", JSONObject().put("rows", rows).put("visibleRowIds", visibleRowIds).put("atEnd", atEnd))
            /** `timeline.minimapIndex` · integer: the marker under y on a rail height tall with count markers, or -1 */
            fun minimapIndex(y: Double, height: Double, count: Long): Any? =
                client.domain("timeline.minimapIndex", JSONObject().put("y", y).put("height", height).put("count", count))
        }

        class ToolInspector internal constructor(private val client: T3Client) {
            /** `toolInspector.compactReference` · object {sourceThreadId, sourceItemId, type, runId, providerTurnId, revision} of a turn item or projected row (revision live while it runs); timeline activity entries already carry it as inspector */
            fun compactReference(row: JSONObject): Any? =
                client.domain("toolInspector.compactReference", JSONObject().put("row", row))
            /** `toolInspector.scope` · object {environment, threadId, sourceThreadId, sourceItemId, type, runId, providerTurnId, revision}: the inspector's local scope, to match the read's answer against */
            fun scope(reference: JSONObject, environment: String, threadId: String): Any? =
                client.domain("toolInspector.scope", JSONObject().put("reference", reference).put("environment", environment).put("threadId", threadId))
            /** `toolInspector.request` · object {method: orchestration.getTurnItem, payload {threadId, itemId, revision}}: the typed call that reads the item */
            fun request(reference: JSONObject, environment: String, threadId: String): Any? =
                client.domain("toolInspector.request", JSONObject().put("reference", reference).put("environment", environment).put("threadId", threadId))
            /** `toolInspector.parse` · object {scope, available, title, status, failed, exitCode, sections [{key, label, text, truncated}], metadata, metadataTruncated, raw, rawTruncated, outputOmitted, emptyOutput} from orchestration.getTurnItem's {item}; {scope, available: false, sections: [], raw: "", rawTruncated: false} when the item is gone */
            fun parse(reference: JSONObject, environment: String, threadId: String, response: JSONObject): Any? =
                client.domain("toolInspector.parse", JSONObject().put("reference", reference).put("environment", environment).put("threadId", threadId).put("response", response))
        }

        class UsageSummary internal constructor(private val client: T3Client) {
            /** `usageSummary.window` · object {sinceDay, untilDay}: the period's days ending `today` (YYYY-MM-DD, the device's local date) */
            fun window(period: String, today: String): Any? =
                client.domain("usageSummary.window", JSONObject().put("period", period).put("today", today))
            /** `usageSummary.request` · object {sinceDay, untilDay, payload}: payload is server.getUsageSummary's arguments, days bucketed in the IANA `timeZone` */
            fun request(period: String, today: String, timeZone: String): Any? =
                client.domain("usageSummary.request", JSONObject().put("period", period).put("today", today).put("timeZone", timeZone))
            /** `usageSummary.aggregate` · object {sinceDay, untilDay, cost, tokens, sessions, cacheSavings, unpricedRecords, kinds {input, cached, cacheWrite, output, reasoning}, days [{day, cost, tokens}], models [{provider, model, cost, tokens, records}] (top 12), providers [{provider, cost, tokens}], sources [{provider, status, message, refreshing}], pricing, readAt} */
            fun aggregate(summary: JSONObject, sinceDay: String, untilDay: String): Any? =
                client.domain("usageSummary.aggregate", JSONObject().put("summary", summary).put("sinceDay", sinceDay).put("untilDay", untilDay))
        }
    }
}
