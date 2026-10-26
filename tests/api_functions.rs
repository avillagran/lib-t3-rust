//! One row per typed function: JSON args → exact tag and payload on the wire,
//! canned response → output. Then the same calls through the Rust signatures,
//! and every local rejection without a single socket.
#[path = "support/fake_t3.rs"]
mod fake_t3;
use fake_t3::{wire, FakeT3, Reply};
use serde_json::{json, Value};
use t3_protocol::api::{
    self, assets::AssetResource, device::*, diagnostics::UsageQuery, orchestration::*, preview::*, project_clone::CloneStart,
    projects::*, provider::AuthResponse, pull_requests::*, review::*, scheduled_tasks::TaskUpsert, secrets::SecretAnswer,
    server::*, terminal::TerminalOpen, vcs::RefQuery,
};

fn launch_args() -> Value {
    json!({"commandId":"omarchy-l1","projectId":"p-1","title":"  Fix the build  ","modelSelection":{"instanceId":"codex","model":"gpt-5"},
        "runtimeMode":"full-access","interactionMode":"default","workspaceStrategy":{"type":"worktree","baseRef":"main","startFromOrigin":true},
        "creationSource":"mobile","initialMessage":{"messageId":"omarchy-m1","text":"hello","attachments":[]}})
}
fn pr() -> Value {
    json!({"projectId":"p-1","repository":"o/r","number":7,"host":"github.com"})
}
fn with(mut base: Value, extra: Value) -> Value {
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}
fn activity_args() -> Value {
    json!({"clientId":"omarchy-android","clientKind":"mobile","visible":true,"focused":false,"recentlyInteracted":true,
        "scopes":[{"type":"server-config"},{"type":"thread","threadId":"t-1"},{"type":"vcs-status","cwd":"/r"},{"type":"provider-status"}],
        "observedAt":"2026-10-24T10:00:00Z","appState":"background","lowPowerMode":"true","batteryState":"unplugged","networkType":" wifi ","ttlMs":60000})
}
fn task_args() -> Value {
    json!({"commandId":"omarchy-s1","title":"Nightly","prompt":"Run the tests","enabled":true,
        "schedule":{"type":"fixed_time","timeOfDay":"9:30","weekdays":[1,2,3,4,5]},"projectId":"p-1","workspaceStrategy":{"type":"root"},
        "modelSelection":{"instanceId":"codex","model":"gpt-5"},"runtimeMode":"full-access","interactionMode":"default","creationSource":"mobile"})
}
fn webhook_task_args() -> Value {
    json!({"id":"scheduled-task:hook-1","title":"Triage","prompt":"Triage {{body.issue}}","enabled":false,
        "schedule":{"type":"webhook","signature":{"header":"X-Hub-Signature-256","encoding":"hex","prefix":"sha256=","secretRef":"sec-1"},
            "maxDeliveryAgeMinutes":null,"dedupe":{"replayKey":"{{headers.x-github-delivery}}"},
            "filters":[{"path":"body.action","op":"in","value":["opened","reopened"]},{"path":"headers.x-github-event","op":"exists"}]},
        "projectId":"p-1","threadId":null,"workspaceStrategy":{"type":"existing_worktree","worktreePath":"/w/t"},
        "modelSelection":{"instanceId":"claude","model":"opus"},"runtimeMode":"approval-required","interactionMode":"plan"})
}
fn mutate_update_args() -> Value {
    json!({"type":"project.update","commandId":"omarchy-p1","projectId":"p-1","title":" Renamed ","defaultThreadEnvMode":null,
        "projectIcon":{"kind":"emoji","emoji":"🚀"},"scripts":[{"id":"build","name":"Build","command":"cargo build","icon":"build","runOnWorktreeCreate":false}]})
}

/// (function, wire tag, args, payload on the wire), in registry order.
fn rows() -> Vec<(&'static str, &'static str, Value, Value)> {
    let mut launch = launch_args();
    launch["title"] = json!("Fix the build");
    let mut activity = activity_args();
    activity["networkType"] = json!("wifi");
    let mut update = mutate_update_args();
    update["title"] = json!("Renamed");
    let pr_with = |extra: Value| with(pr(), extra);
    vec![
        // server
        ("server.getConfig", "server.getConfig", json!({}), json!({})),
        ("server.getSettings", "server.getSettings", json!({}), json!({})),
        ("server.updateSettings", "server.updateSettings", json!({"patch":{"sidebarAutoSettleAfterDays":3}}), json!({"patch":{"sidebarAutoSettleAfterDays":3}})),
        (
            "server.updateSettings",
            "server.updateSettings",
            json!({"patch":{},"providerInstanceMutation":{"operation":"remove","instanceId":"codex_2"}}),
            json!({"patch":{},"providerInstanceMutation":{"operation":"remove","instanceId":"codex_2"}}),
        ),
        ("server.discoverSourceControl", "server.discoverSourceControl", json!({}), json!({})),
        ("server.reportClientActivity", "server.reportClientActivity", activity_args(), activity),
        (
            "server.upsertKeybinding",
            "server.upsertKeybinding",
            json!({"key":" mod+shift+k ","command":"thread.next","when":"!terminalFocus","replace":{"key":"mod+k","command":"thread.next"}}),
            json!({"key":"mod+shift+k","command":"thread.next","when":"!terminalFocus","replace":{"key":"mod+k","command":"thread.next"}}),
        ),
        ("server.removeKeybinding", "server.removeKeybinding", json!({"key":"mod+k","command":"script.build.run"}), json!({"key":"mod+k","command":"script.build.run"})),
        ("server.searchAcpRegistry", "server.searchAcpRegistry", json!({"query":"  ","limit":20}), json!({"query":"","limit":20})),
        ("server.prepareAcpRegistryAgent", "server.prepareAcpRegistryAgent", json!({"agentId":"claude-code.acp"}), json!({"agentId":"claude-code.acp"})),
        // orchestration
        (
            "orchestration.dispatchCommand",
            "orchestration.dispatchCommand",
            json!({"type":"thread.archive","commandId":"omarchy-1","threadId":"t-1"}),
            json!({"type":"thread.archive","commandId":"omarchy-1","threadId":"t-1"}),
        ),
        ("orchestration.launchThread", "orchestration.launchThread", launch_args(), launch),
        ("orchestration.getThreadProjection", "orchestration.getThreadProjection", json!({"threadId":"t-1"}), json!({"threadId":"t-1"})),
        (
            "orchestration.getTurnItem",
            "orchestration.getTurnItem",
            json!({"threadId":"t-1","itemId":"item-1","revision":"r3"}),
            json!({"threadId":"t-1","itemId":"item-1","revision":"r3"}),
        ),
        (
            "orchestration.getTurnDiff",
            "orchestration.getTurnDiff",
            json!({"threadId":"t-1","fromTurnCount":1,"toTurnCount":3,"ignoreWhitespace":true}),
            json!({"threadId":"t-1","fromTurnCount":1,"toTurnCount":3,"ignoreWhitespace":true}),
        ),
        ("orchestration.getFullThreadDiff", "orchestration.getFullThreadDiff", json!({"threadId":"t-1","toTurnCount":3}), json!({"threadId":"t-1","toTurnCount":3})),
        (
            "orchestration.getWorkflowScript",
            "orchestration.getWorkflowScript",
            json!({"threadId":"t-1","scriptPath":"/w/scripts/a.js"}),
            json!({"threadId":"t-1","scriptPath":"/w/scripts/a.js"}),
        ),
        ("orchestration.stopWorkflow", "orchestration.stopWorkflow", json!({"threadId":"t-1","subagentId":"node-1"}), json!({"threadId":"t-1","subagentId":"node-1"})),
        ("orchestration.searchThreads", "orchestration.searchThreads", json!({"query":"  build failure ","limit":10}), json!({"query":"build failure","limit":10})),
        ("orchestration.getArchivedShellSnapshot", "orchestration.getArchivedShellSnapshot", json!({}), json!({})),
        // preview
        ("preview.list", "preview.list", json!({"threadId":"t-1"}), json!({"threadId":"t-1"})),
        (
            "preview.open",
            "preview.open",
            json!({"threadId":"t-1","url":" http://localhost:3000 ","viewport":{"_tag":"preset","width":390,"height":844,"presetId":"iphone-12-pro"},"profileId":"default","runtime":"server","reveal":true}),
            json!({"threadId":"t-1","url":"http://localhost:3000","viewport":{"_tag":"preset","width":390,"height":844,"presetId":"iphone-12-pro"},"profileId":"default","runtime":"server","reveal":true}),
        ),
        (
            "preview.navigate",
            "preview.navigate",
            json!({"threadId":"t-1","tabId":"tab-1","url":"http://localhost:3000/a","resolvedTitle":""}),
            json!({"threadId":"t-1","tabId":"tab-1","url":"http://localhost:3000/a","resolvedTitle":""}),
        ),
        (
            "preview.resize",
            "preview.resize",
            json!({"threadId":"t-1","tabId":"tab-1","viewport":{"_tag":"freeform","width":1280,"height":800}}),
            json!({"threadId":"t-1","tabId":"tab-1","viewport":{"_tag":"freeform","width":1280,"height":800}}),
        ),
        (
            "preview.adjust",
            "preview.adjust",
            json!({"threadId":"t-1","tabId":"tab-1","colorScheme":"dark","zoomFactor":1.25,"hardReload":false,"clear":"cache"}),
            json!({"threadId":"t-1","tabId":"tab-1","colorScheme":"dark","zoomFactor":1.25,"hardReload":false,"clear":"cache"}),
        ),
        ("preview.refresh", "preview.refresh", json!({"threadId":"t-1","tabId":"tab-1"}), json!({"threadId":"t-1","tabId":"tab-1"})),
        ("preview.close", "preview.close", json!({"threadId":"t-1"}), json!({"threadId":"t-1"})),
        ("preview.clearProfile", "preview.clearProfile", json!({"profileId":"work"}), json!({"profileId":"work"})),
        (
            "preview.reportStatus",
            "preview.reportStatus",
            json!({"threadId":"t-1","tabId":"tab-1","navStatus":{"_tag":"LoadFailed","url":"http://x.test","title":"","code":-105,"description":"DNS"},"canGoBack":false,"canGoForward":true}),
            json!({"threadId":"t-1","tabId":"tab-1","navStatus":{"_tag":"LoadFailed","url":"http://x.test","title":"","code":-105,"description":"DNS"},"canGoBack":false,"canGoForward":true}),
        ),
        // assets
        (
            "assets.createUrl",
            "assets.createUrl",
            json!({"resource":{"_tag":"workspace-file","threadId":"t-1","path":"README.md"}}),
            json!({"resource":{"_tag":"workspace-file","threadId":"t-1","path":"README.md"}}),
        ),
        ("attachments.delete", "attachments.delete", json!({"attachmentId":"att-1"}), json!({"attachmentId":"att-1"})),
        // vcs
        ("vcs.refreshStatus", "vcs.refreshStatus", json!({"cwd":"/r"}), json!({"cwd":"/r"})),
        (
            "vcs.listRefs",
            "vcs.listRefs",
            json!({"cwd":"/r","query":" feat ","refKind":"local","limit":50,"cursor":0}),
            json!({"cwd":"/r","query":"feat","refKind":"local","limit":50,"cursor":0}),
        ),
        ("vcs.createRef", "vcs.createRef", json!({"cwd":"/r","refName":" feat/x ","switchRef":true}), json!({"cwd":"/r","refName":"feat/x","switchRef":true})),
        ("vcs.switchRef", "vcs.switchRef", json!({"cwd":"/r","refName":"main"}), json!({"cwd":"/r","refName":"main"})),
        ("vcs.pull", "vcs.pull", json!({"cwd":"/r"}), json!({"cwd":"/r"})),
        ("vcs.init", "vcs.init", json!({"cwd":"/r","kind":"git"}), json!({"cwd":"/r","kind":"git"})),
        (
            "git.preparePullRequestThread",
            "git.preparePullRequestThread",
            json!({"cwd":"/r","reference":"#42","mode":"worktree","threadId":"t-1"}),
            json!({"cwd":"/r","reference":"#42","mode":"worktree","threadId":"t-1"}),
        ),
        // review
        (
            "review.getDiffPreview",
            "review.getDiffPreview",
            json!({"cwd":"/r","baseRef":" main ","ignoreWhitespace":true,"file":{"path":"src/a.rs","previousPath":null,"sourceKind":"branch-range"}}),
            json!({"cwd":"/r","baseRef":"main","ignoreWhitespace":true,"file":{"path":"src/a.rs","previousPath":null,"sourceKind":"branch-range"}}),
        ),
        (
            "review.getDiffFileContents",
            "review.getDiffFileContents",
            json!({"cwd":"/r","sourceKind":"branch-range","changeType":"rename-changed","baseRef":"main","oldPath":"a.rs","newPath":"b.rs"}),
            json!({"cwd":"/r","sourceKind":"branch-range","changeType":"rename-changed","baseRef":"main","headRef":null,"oldPath":"a.rs","newPath":"b.rs"}),
        ),
        // projects
        (
            "projects.readFile",
            "projects.readFile",
            json!({"cwd":"/r","relativePath":"docs/x.md","allowMissing":true}),
            json!({"cwd":"/r","relativePath":"docs/x.md","allowMissing":true}),
        ),
        (
            "projects.searchEntries",
            "projects.searchEntries",
            json!({"cwd":"/r","query":"","limit":40,"kind":"file","imageOnly":true}),
            json!({"cwd":"/r","query":"","limit":40,"kind":"file","imageOnly":true}),
        ),
        ("projects.mutate", "projects.mutate", mutate_update_args(), update),
        (
            "projects.mutate",
            "projects.mutate",
            json!({"type":"project.create","commandId":"omarchy-p2","projectId":"p-2","title":"New","workspaceRoot":"/home/u/new","createWorkspaceRootIfMissing":true}),
            json!({"type":"project.create","commandId":"omarchy-p2","projectId":"p-2","title":"New","workspaceRoot":"/home/u/new","createWorkspaceRootIfMissing":true}),
        ),
        (
            "projects.mutate",
            "projects.mutate",
            json!({"type":"project.delete","commandId":"omarchy-p3","projectId":"p-3","force":true}),
            json!({"type":"project.delete","commandId":"omarchy-p3","projectId":"p-3","force":true}),
        ),
        ("projects.createNew", "projects.createNew", json!({"name":"  demo  "}), json!({"name":"demo"})),
        // diagnostics
        ("server.getHostResources", "server.getHostResources", json!({}), json!({})),
        ("server.getProcessDiagnostics", "server.getProcessDiagnostics", json!({}), json!({})),
        ("server.getResourceTelemetryHistory", "server.getResourceTelemetryHistory", json!({"windowMs":3600000,"bucketMs":60000}), json!({"windowMs":3600000,"bucketMs":60000})),
        (
            "server.signalProcess",
            "server.signalProcess",
            json!({"pid":4242,"startTimeMs":1700000000000u64,"signal":"SIGINT"}),
            json!({"pid":4242,"startTimeMs":1700000000000u64,"signal":"SIGINT"}),
        ),
        (
            "server.checkOtlpEndpoint",
            "server.checkOtlpEndpoint",
            json!({"signal":"traces","url":" https://otel.example/v1/traces "}),
            json!({"signal":"traces","url":"https://otel.example/v1/traces"}),
        ),
        (
            "server.getUsageSummary",
            "server.getUsageSummary",
            json!({"sinceDay":"2026-10-01","untilDay":"2026-10-24","timeZone":"America/Santiago","resolution":"day","awaitRefresh":true}),
            json!({"sinceDay":"2026-10-01","untilDay":"2026-10-24","timeZone":"America/Santiago","resolution":"day","awaitRefresh":true}),
        ),
        ("server.refreshUsageRates", "server.refreshUsageRates", json!({}), json!({})),
        ("server.runStorageCleanup", "server.runStorageCleanup", json!({}), json!({})),
        // provider
        (
            "server.refreshProviders",
            "server.refreshProviders",
            json!({"instanceId":"codex","cwd":"/r","refreshModels":true}),
            json!({"instanceId":"codex","cwd":"/r","refreshModels":true}),
        ),
        ("server.updateProvider", "server.updateProvider", json!({"provider":"codex","targetVersion":"0.50.0"}), json!({"provider":"codex","targetVersion":"0.50.0"})),
        (
            "provider.auth.start",
            "provider.auth.start",
            json!({"instanceId":"claude","methodId":"oauth","callbackMode":"client"}),
            json!({"instanceId":"claude","methodId":"oauth","callbackMode":"client"}),
        ),
        (
            "provider.auth.respond",
            "provider.auth.respond",
            json!({"instanceId":"claude","flowId":"f-1","interactionId":"i-1","response":{"type":"terminal","data":"","size":{"cols":80,"rows":24}}}),
            json!({"instanceId":"claude","flowId":"f-1","interactionId":"i-1","response":{"type":"terminal","data":"","size":{"cols":80,"rows":24}}}),
        ),
        (
            "provider.auth.respond",
            "provider.auth.respond",
            json!({"instanceId":"claude","flowId":"f-1","interactionId":"i-2","response":{"type":"credentials","values":{"apiKey":"synthetic-value"}}}),
            json!({"instanceId":"claude","flowId":"f-1","interactionId":"i-2","response":{"type":"credentials","values":{"apiKey":"synthetic-value"}}}),
        ),
        ("provider.auth.cancel", "provider.auth.cancel", json!({"instanceId":"claude","flowId":"f-1"}), json!({"instanceId":"claude","flowId":"f-1"})),
        ("provider.auth.logout", "provider.auth.logout", json!({"instanceId":"claude"}), json!({"instanceId":"claude"})),
        ("provider.install.start", "provider.install.start", json!({"instanceId":"opencode"}), json!({"instanceId":"opencode"})),
        ("provider.install.cancel", "provider.install.cancel", json!({"instanceId":"opencode","operationId":"op-1"}), json!({"instanceId":"opencode","operationId":"op-1"})),
        ("provider.install.remove", "provider.install.remove", json!({"instanceId":"opencode"}), json!({"instanceId":"opencode"})),
        // projectClone
        (
            "projectClone.start",
            "projectClone.start",
            json!({"projectId":"p-9","title":"Repo","createdAt":"2026-10-24T10:00:00.000Z","destinationPath":"/home/u/repo","provider":"github","repository":"o/r","protocol":"ssh"}),
            json!({"projectId":"p-9","title":"Repo","createdAt":"2026-10-24T10:00:00.000Z","destinationPath":"/home/u/repo","provider":"github","repository":"o/r","protocol":"ssh"}),
        ),
        // filesystem
        ("filesystem.browse", "filesystem.browse", json!({"partialPath":"~/Wo","cwd":"/home/u"}), json!({"partialPath":"~/Wo","cwd":"/home/u"})),
        ("filesystem.getMetadata", "filesystem.getMetadata", json!({"paths":["/home/u/a.png","~/b"]}), json!({"paths":["/home/u/a.png","~/b"]})),
        // pullRequests
        (
            "pullRequests.list",
            "pullRequests.list",
            json!({"state":"open","involvement":"authored","filters":{"draft":"hide","labels":[["bug"]]},"limit":50,"query":"rust","cursors":{"github.com":"abc"}}),
            json!({"state":"open","involvement":"authored","filters":{"draft":"hide","labels":[["bug"]]},"limit":50,"query":"rust","cursors":{"github.com":"abc"}}),
        ),
        ("pullRequests.detail", "pullRequests.detail", pr(), pr()),
        ("pullRequests.checks", "pullRequests.checks", pr_with(json!({"allowStale":true})), pr_with(json!({"allowStale":true}))),
        ("pullRequests.activity", "pullRequests.activity", pr(), pr()),
        ("pullRequests.routing", "pullRequests.routing", pr_with(json!({"expectedAccountId":"acct-1"})), pr_with(json!({"expectedAccountId":"acct-1"}))),
        ("pullRequests.routingIdentity", "pullRequests.routingIdentity", json!({"host":"github.com"}), json!({"host":"github.com"})),
        ("pullRequests.reviewerCandidates", "pullRequests.reviewerCandidates", pr(), pr()),
        ("pullRequests.labelCandidates", "pullRequests.labelCandidates", pr(), pr()),
        ("pullRequests.comment", "pullRequests.comment", pr_with(json!({"body":" "})), pr_with(json!({"body":" "}))),
        (
            "pullRequests.submitReview",
            "pullRequests.submitReview",
            pr_with(json!({"verdict":"request-changes","body":"","comments":[{"path":"src/a.rs","body":"nit","position":{"kind":"context","oldLine":3,"newLine":4,"side":"right"}}]})),
            pr_with(json!({"verdict":"request-changes","body":"","comments":[{"path":"src/a.rs","body":"nit","position":{"kind":"context","oldLine":3,"newLine":4,"side":"right"}}]})),
        ),
        (
            "pullRequests.runAction",
            "pullRequests.runAction",
            pr_with(json!({"action":"merge","mergeMethod":"squash","stackNumber":2,"expectedStackHeads":[{"number":7,"headSha":"abc123"}]})),
            pr_with(json!({"action":"merge","mergeMethod":"squash","stackNumber":2,"expectedStackHeads":[{"number":7,"headSha":"abc123"}]})),
        ),
        (
            "pullRequests.requestReviewers",
            "pullRequests.requestReviewers",
            pr_with(json!({"reviewers":[{"id":"octocat","kind":"user"}],"requested":true})),
            pr_with(json!({"reviewers":[{"id":"octocat","kind":"user"}],"requested":true})),
        ),
        ("pullRequests.setLabels", "pullRequests.setLabels", pr_with(json!({"labels":["bug"],"applied":false})), pr_with(json!({"labels":["bug"],"applied":false}))),
        // device
        ("device.list", "device.list", json!({"updateTool":"hub"}), json!({"updateTool":"hub"})),
        ("device.configure", "device.configure", json!({"enabled":true,"onboardingCompleted":true}), json!({"enabled":true,"onboardingCompleted":true})),
        (
            "device.testHost",
            "device.testHost",
            json!({"id":"mac-mini","label":"Mac mini","target":"me@mac.local","port":22}),
            json!({"kind":"ssh","id":"mac-mini","label":"Mac mini","target":"me@mac.local","port":22}),
        ),
        (
            "device.open",
            "device.open",
            json!({"threadId":"t-1","deviceId":"emulator-5554","platform":"android","boot":true}),
            json!({"threadId":"t-1","deviceId":"emulator-5554","platform":"android","boot":true}),
        ),
        (
            "device.close",
            "device.close",
            json!({"threadId":"t-1","deviceId":"emulator-5554","shutdown":true}),
            json!({"threadId":"t-1","deviceId":"emulator-5554","shutdown":true}),
        ),
        ("device.detail", "device.detail", json!({"deviceId":"emulator-5554","hostId":"local"}), json!({"deviceId":"emulator-5554","hostId":"local"})),
        (
            "device.action",
            "device.action",
            json!({"type":"setLocation","deviceId":"emulator-5554","latitude":-33.45,"longitude":-70.66}),
            json!({"type":"setLocation","deviceId":"emulator-5554","latitude":-33.45,"longitude":-70.66}),
        ),
        (
            "device.action",
            "device.action",
            json!({"type":"sendPush","deviceId":"booted","hostId":"mac-mini","appId":"com.example","payload":{"aps":{"alert":"hi"}}}),
            json!({"type":"sendPush","deviceId":"booted","hostId":"mac-mini","appId":"com.example","payload":{"aps":{"alert":"hi"}}}),
        ),
        // scheduledTasks
        ("scheduledTasks.list", "scheduledTasks.list", json!({}), json!({})),
        ("scheduledTasks.upsert", "scheduledTasks.upsert", task_args(), task_args()),
        ("scheduledTasks.upsert", "scheduledTasks.upsert", webhook_task_args(), webhook_task_args()),
        ("scheduledTasks.setEnabled", "scheduledTasks.setEnabled", json!({"id":"scheduled-task:omarchy-s1","enabled":false}), json!({"id":"scheduled-task:omarchy-s1","enabled":false})),
        ("scheduledTasks.delete", "scheduledTasks.delete", json!({"id":"scheduled-task:omarchy-s1"}), json!({"id":"scheduled-task:omarchy-s1"})),
        ("scheduledTasks.runNow", "scheduledTasks.runNow", json!({"id":"scheduled-task:omarchy-s1"}), json!({"id":"scheduled-task:omarchy-s1"})),
        // agentSessions
        ("agentSessions.scan", "agentSessions.scan", json!({}), json!({})),
        ("agentSessions.import", "agentSessions.import", json!({"projectId":"p-1","expectedWorkspaceRoot":"/r"}), json!({"projectId":"p-1","expectedWorkspaceRoot":"/r"})),
        // mcpApps
        (
            "mcpApps.callTool",
            "mcpApps.callTool",
            json!({"threadId":"t-1","itemId":"item-1","name":"get_weather","arguments":{"city":"Santiago"}}),
            json!({"threadId":"t-1","itemId":"item-1","name":"get_weather","arguments":{"city":"Santiago"}}),
        ),
        (
            "mcpApps.readResource",
            "mcpApps.readResource",
            json!({"threadId":"t-1","itemId":"item-1","uri":"ui://weather/app.html"}),
            json!({"threadId":"t-1","itemId":"item-1","uri":"ui://weather/app.html"}),
        ),
        (
            "mcpApps.updateModelContext",
            "mcpApps.updateModelContext",
            json!({"threadId":"t-1","itemId":"item-1","conversationThreadId":"t-1","content":[{"type":"text","text":"72F"}]}),
            json!({"threadId":"t-1","itemId":"item-1","conversationThreadId":"t-1","content":[{"type":"text","text":"72F"}]}),
        ),
        // shell, secrets, worktreeSetup
        ("shell.openInEditor", "shell.openInEditor", json!({"cwd":"/r","editor":"zed"}), json!({"cwd":"/r","editor":"zed"})),
        (
            "secrets.answerRequest",
            "secrets.answerRequest",
            json!({"threadId":"t-1","turnItemId":"item-2","answer":{"type":"decline"}}),
            json!({"threadId":"t-1","turnItemId":"item-2","answer":{"type":"decline"}}),
        ),
        ("worktreeSetup.cancel", "worktreeSetup.cancel", json!({"threadId":"t-1"}), json!({"threadId":"t-1"})),
        // terminal
        (
            "terminal.open",
            "terminal.open",
            json!({"threadId":"t-1","terminalId":"term-1","cwd":"/r","worktreePath":null,"cols":120,"rows":40,"env":{"TERM":"xterm-256color"},"providerInstanceId":"codex"}),
            json!({"threadId":"t-1","terminalId":"term-1","cwd":"/r","worktreePath":null,"cols":120,"rows":40,"env":{"TERM":"xterm-256color"},"providerInstanceId":"codex"}),
        ),
        ("terminal.write", "terminal.write", json!({"threadId":"t-1","terminalId":"term-1","data":" ls\r"}), json!({"threadId":"t-1","terminalId":"term-1","data":" ls\r"})),
        ("terminal.resize", "terminal.resize", json!({"threadId":"t-1","terminalId":"term-1","cols":100,"rows":30}), json!({"threadId":"t-1","terminalId":"term-1","cols":100,"rows":30})),
        ("orchestration.getThreadFamily", "orchestration.getThreadFamily", json!({"threadId":"t-1"}), json!({"threadId":"t-1"})),
        ("orchestration.getWorktreeThreadIds", "orchestration.getWorktreeThreadIds", json!({"worktreePath":"/w/t"}), json!({"worktreePath":"/w/t"})),
        ("orchestration.getOlderSettledThreads", "orchestration.getOlderSettledThreads", json!({"before":{"settledAt":"2026-10-26T00:00:00Z"},"limit":50}), json!({"before":{"settledAt":"2026-10-26T00:00:00Z"},"limit":50})),
        ("guestShare.create", "guestShare.create", json!({"threadId":"t-1"}), json!({"threadId":"t-1"})),
        ("projects.listIgnoredFiles", "projects.listIgnoredFiles", json!({"cwd":"/r"}), json!({"cwd":"/r"})),
        ("device.easAccount", "device.easAccount", json!({"hostId":"eas1"}), json!({"hostId":"eas1"})),
        ("pullRequests.changeBreakdown", "pullRequests.changeBreakdown", json!({"projectId":"p-1","repository":"o/r","number":7,"host":"github.com"}), json!({"projectId":"p-1","repository":"o/r","number":7,"host":"github.com"})),
        ("pullRequests.filterOptions", "pullRequests.filterOptions", json!({"kind":"labels","projectId":"p-1","query":"bug"}), json!({"kind":"labels","projectId":"p-1","query":"bug"})),
    ]
}
fn expected(rows: &[(&str, &str, Value, Value)]) -> Vec<Value> {
    rows.iter().map(|(_, tag, _, payload)| json!({"tag":tag,"payload":payload})).collect()
}

#[test]
fn every_function_sends_its_exact_tag_and_payload_once() {
    let rows = rows();
    let fake = FakeT3::spawn(rows.iter().enumerate().map(|(i, _)| if i % 7 == 3 { Reply::Ok(Value::Null) } else { Reply::Ok(json!({"row":i})) }).collect());
    let conn = fake.connection();
    for (i, (name, _, args, _)) in rows.iter().enumerate() {
        let out = if i % 7 == 3 { Value::Null } else { json!({"row":i}) };
        assert_eq!(api::call(&conn, name, args, None).unwrap(), out, "{name}");
    }
    let recorded = fake.finish();
    assert_eq!(wire(&recorded), expected(&rows));
    assert_eq!(recorded.connections, rows.len(), "one socket per call, no retries");
    // Every registered unary function has a row (streams: tests/api_streams.rs).
    let mut covered = rows.iter().map(|(name, _, _, _)| *name).collect::<Vec<_>>();
    covered.sort();covered.dedup();
    let unary = api::functions().filter(|f| !f.stream && f.tag.is_some());
    let mut registered=unary.map(|f|f.name).collect::<Vec<_>>();registered.sort();assert_eq!(covered,registered);
    for (name, tag, _, _) in &rows {
        assert_eq!(api::function(name).unwrap().tag, Some(*tag), "{name}");
    }
}

#[test]
fn rust_signatures_put_the_same_requests_on_the_wire() {
    let rows = rows();
    let fake = FakeT3::spawn((0..rows.len()).map(|_| Reply::Ok(json!({"ok":true}))).collect());
    let conn = fake.connection();
    let c = &conn;
    let s = |v: &str| v.to_string();
    // server
    api::server::config(c).unwrap();
    api::server::settings(c).unwrap();
    api::server::update_settings(c, &json!({"sidebarAutoSettleAfterDays":3}), None).unwrap();
    api::server::update_settings(c, &json!({}), Some(&ProviderInstanceMutation::Remove { instance_id: s("codex_2") })).unwrap();
    api::server::discover_source_control(c).unwrap();
    let activity = ClientActivity {
        client_id: s("omarchy-android"),
        client_kind: s("mobile"),
        visible: true,
        focused: false,
        recently_interacted: true,
        scopes: vec![json!({"type":"server-config"}), json!({"type":"thread","threadId":"t-1"}), json!({"type":"vcs-status","cwd":"/r"}), json!({"type":"provider-status"})],
        observed_at: s("2026-10-24T10:00:00Z"),
        environment_id: None,
        app_state: Some(s("background")),
        low_power_mode: Some(s("true")),
        battery_state: Some(s("unplugged")),
        network_type: Some(s(" wifi ")),
        ttl_ms: Some(60000),
    };
    api::server::report_client_activity(c, &activity).unwrap();
    let binding = Keybinding { key: s(" mod+shift+k "), command: s("thread.next"), when: Some(s("!terminalFocus")) };
    api::server::upsert_keybinding(c, &binding, Some(&Keybinding { key: s("mod+k"), command: s("thread.next"), when: None })).unwrap();
    api::server::remove_keybinding(c, &Keybinding { key: s("mod+k"), command: s("script.build.run"), when: None }).unwrap();
    api::server::search_acp_registry(c, "  ", Some(20)).unwrap();
    api::server::prepare_acp_registry_agent(c, "claude-code.acp").unwrap();
    // orchestration
    api::orchestration::dispatch(c, &Command::ThreadArchive { command_id: s("omarchy-1"), thread_id: s("t-1") }).unwrap();
    let launch = ThreadLaunch {
        command_id: s("omarchy-l1"),
        project_id: s("p-1"),
        title: s("  Fix the build  "),
        model_selection: json!({"instanceId":"codex","model":"gpt-5"}),
        runtime_mode: s("full-access"),
        interaction_mode: s("default"),
        workspace_strategy: WorkspaceStrategy::Worktree { base_ref: s("main"), branch: None, start_from_origin: Some(true) },
        creation_source: Some(s("mobile")),
        thread_id: None,
        reuse_existing_thread: None,
        generate_title: None,
        initial_message: Some(InitialMessage { message_id: Some(s("omarchy-m1")), text: s("hello"), context: None, attachments: vec![] }),
    };
    api::orchestration::launch_thread(c, &launch).unwrap();
    api::orchestration::thread_projection(c, "t-1").unwrap();
    api::orchestration::turn_item(c, "t-1", "item-1", Some("r3")).unwrap();
    api::orchestration::turn_diff(c, "t-1", 1, 3, Some(true)).unwrap();
    api::orchestration::full_thread_diff(c, "t-1", 3, None).unwrap();
    api::orchestration::workflow_script(c, "t-1", "/w/scripts/a.js").unwrap();
    api::orchestration::stop_workflow(c, "t-1", "node-1").unwrap();
    api::orchestration::search_threads(c, "  build failure ", Some(10)).unwrap();
    api::orchestration::archived_shell(c).unwrap();
    // preview
    api::preview::list(c, "t-1").unwrap();
    let preview_open = PreviewOpen {
        url: Some(s(" http://localhost:3000 ")),
        viewport: Some(json!({"_tag":"preset","width":390,"height":844,"presetId":"iphone-12-pro"})),
        profile_id: Some(s("default")),
        runtime: Some(s("server")),
        reveal: Some(true),
    };
    api::preview::open(c, "t-1", &preview_open).unwrap();
    api::preview::navigate(c, "t-1", "tab-1", "http://localhost:3000/a", Some("")).unwrap();
    api::preview::resize(c, "t-1", "tab-1", &json!({"_tag":"freeform","width":1280,"height":800})).unwrap();
    let adjust = PreviewAdjust { color_scheme: Some(s("dark")), zoom_factor: Some(1.25), hard_reload: Some(false), clear: Some(s("cache")) };
    api::preview::adjust(c, "t-1", "tab-1", &adjust).unwrap();
    api::preview::refresh(c, "t-1", "tab-1").unwrap();
    api::preview::close(c, "t-1", None).unwrap();
    api::preview::clear_profile(c, "work").unwrap();
    let nav = json!({"_tag":"LoadFailed","url":"http://x.test","title":"","code":-105,"description":"DNS"});
    api::preview::report_status(c, "t-1", "tab-1", &nav, false, true).unwrap();
    // assets
    api::assets::create_url(c, &AssetResource::WorkspaceFile { thread_id: s("t-1"), path: s("README.md") }).unwrap();
    api::assets::delete_attachment(c, "att-1").unwrap();
    // vcs
    api::vcs::refresh_status(c, "/r").unwrap();
    let refs = RefQuery { query: Some(s(" feat ")), ref_kind: Some(s("local")), limit: Some(50), cursor: Some(0), ..RefQuery::default() };
    api::vcs::list_refs(c, "/r", &refs).unwrap();
    api::vcs::create_ref(c, "/r", " feat/x ", Some(true)).unwrap();
    api::vcs::switch_ref(c, "/r", "main").unwrap();
    api::vcs::pull(c, "/r").unwrap();
    api::vcs::init(c, "/r", Some("git")).unwrap();
    api::vcs::prepare_pull_request_thread(c, "/r", "#42", "worktree", Some("t-1")).unwrap();
    // review
    let file = DiffFile { path: s("src/a.rs"), previous_path: None, source_kind: SourceKind::BranchRange };
    api::review::diff_preview(c, "/r", Some(" main "), Some(true), Some(&file)).unwrap();
    let contents = FileContentsQuery {
        cwd: s("/r"),
        source_kind: SourceKind::BranchRange,
        change_type: s("rename-changed"),
        base_ref: Some(s("main")),
        head_ref: None,
        old_path: s("a.rs"),
        new_path: s("b.rs"),
    };
    api::review::diff_file_contents(c, &contents).unwrap();
    // projects
    api::projects::read_file(c, "/r", "docs/x.md", Some(true)).unwrap();
    api::projects::search_entries(c, "/r", "", 40, Some("file"), Some(true)).unwrap();
    let update = ProjectUpdate {
        title: Some(s(" Renamed ")),
        default_thread_env_mode: Some(None),
        project_icon: Some(json!({"kind":"emoji","emoji":"🚀"})),
        scripts: Some(vec![json!({"id":"build","name":"Build","command":"cargo build","icon":"build","runOnWorktreeCreate":false})]),
        ..ProjectUpdate::default()
    };
    api::projects::mutate(c, &ProjectMutation::Update { command_id: s("omarchy-p1"), project_id: s("p-1"), update }).unwrap();
    let create = ProjectMutation::Create {
        command_id: s("omarchy-p2"),
        project_id: s("p-2"),
        title: s("New"),
        workspace_root: s("/home/u/new"),
        create_workspace_root_if_missing: Some(true),
        default_model_selection: None,
        scripts: None,
    };
    api::projects::mutate(c, &create).unwrap();
    api::projects::mutate(c, &ProjectMutation::Delete { command_id: s("omarchy-p3"), project_id: s("p-3"), force: Some(true) }).unwrap();
    api::projects::create_new(c, "  demo  ").unwrap();
    // diagnostics
    api::diagnostics::host_resources(c).unwrap();
    api::diagnostics::processes(c).unwrap();
    api::diagnostics::telemetry_history(c, 3_600_000, 60_000).unwrap();
    api::diagnostics::signal_process(c, 4242, 1_700_000_000_000, "SIGINT").unwrap();
    api::diagnostics::check_otlp_endpoint(c, "traces", " https://otel.example/v1/traces ").unwrap();
    let usage = UsageQuery {
        since_day: s("2026-10-01"),
        until_day: s("2026-10-24"),
        time_zone: s("America/Santiago"),
        resolution: Some(s("day")),
        await_refresh: Some(true),
        ..UsageQuery::default()
    };
    api::diagnostics::usage_summary(c, &usage).unwrap();
    api::diagnostics::refresh_usage_rates(c).unwrap();
    api::diagnostics::run_storage_cleanup(c).unwrap();
    // provider
    api::provider::refresh(c, Some("codex"), Some("/r"), None, Some(true)).unwrap();
    api::provider::update(c, "codex", Some("0.50.0"), None).unwrap();
    api::provider::auth_start(c, "claude", Some("oauth"), None, Some("client")).unwrap();
    api::provider::auth_respond(c, "claude", "f-1", "i-1", &AuthResponse::Terminal { data: s(""), size: Some((80, 24)) }).unwrap();
    let mut values = serde_json::Map::new();
    values.insert(s("apiKey"), json!("synthetic-value"));
    let credentials = AuthResponse::Credentials { values };
    assert!(!format!("{credentials:?}").contains("synthetic-value"), "Debug never shows a credential");
    api::provider::auth_respond(c, "claude", "f-1", "i-2", &credentials).unwrap();
    api::provider::auth_cancel(c, "claude", "f-1").unwrap();
    api::provider::auth_logout(c, "claude").unwrap();
    api::provider::install_start(c, "opencode").unwrap();
    api::provider::install_cancel(c, "opencode", "op-1").unwrap();
    api::provider::install_remove(c, "opencode").unwrap();
    // projectClone, filesystem
    let clone = CloneStart {
        project_id: s("p-9"),
        title: s("Repo"),
        created_at: s("2026-10-24T10:00:00.000Z"),
        destination_path: s("/home/u/repo"),
        provider: Some(s("github")),
        repository: Some(s("o/r")),
        remote_url: None,
        protocol: Some(s("ssh")),
    };
    api::project_clone::start(c, &clone).unwrap();
    api::filesystem::browse(c, "~/Wo", Some("/home/u")).unwrap();
    api::filesystem::metadata(c, &[s("/home/u/a.png"), s("~/b")]).unwrap();
    // pullRequests
    let mut list = ListQuery::new("open");
    list.involvement = Some(s("authored"));
    list.filters = Some(json!({"draft":"hide","labels":[["bug"]]}));
    list.limit = Some(50);
    list.query = Some(s("rust"));
    list.cursors = json!({"github.com":"abc"}).as_object().cloned();
    api::pull_requests::list(c, &list).unwrap();
    let mut pr = PrRef::new("p-1", "o/r", 7);
    pr.host = Some(s("github.com"));
    api::pull_requests::detail(c, &pr).unwrap();
    api::pull_requests::checks(c, &PrRef { allow_stale: Some(true), ..pr.clone() }).unwrap();
    api::pull_requests::activity(c, &pr).unwrap();
    api::pull_requests::routing(c, &PrRef { expected_account_id: Some(s("acct-1")), ..pr.clone() }).unwrap();
    api::pull_requests::routing_identity(c, "github.com").unwrap();
    api::pull_requests::reviewer_candidates(c, &pr).unwrap();
    api::pull_requests::label_candidates(c, &pr).unwrap();
    api::pull_requests::comment(c, &pr, " ").unwrap();
    let comment = json!({"path":"src/a.rs","body":"nit","position":{"kind":"context","oldLine":3,"newLine":4,"side":"right"}});
    api::pull_requests::submit_review(c, &pr, "request-changes", "", &[comment]).unwrap();
    let action = PrAction { action: s("merge"), merge_method: Some(s("squash")), stack_number: Some(2), expected_stack_heads: Some(vec![(7, s("abc123"))]), ..PrAction::default() };
    api::pull_requests::run_action(c, &pr, &action).unwrap();
    api::pull_requests::request_reviewers(c, &pr, &[(s("octocat"), s("user"))], true).unwrap();
    api::pull_requests::set_labels(c, &pr, &[s("bug")], false).unwrap();
    // device
    api::device::list(c, Some("hub"), None, None).unwrap();
    api::device::configure(c, Some(true), None, Some(true)).unwrap();
    api::device::test_host(c, &SshHost { id: s("mac-mini"), label: s("Mac mini"), target: s("me@mac.local"), identity_file: None, port: Some(22) }).unwrap();
    api::device::open(c, "t-1", "emulator-5554", "android", None, Some(true)).unwrap();
    api::device::close(c, "t-1", None, Some("emulator-5554"), Some(true)).unwrap();
    api::device::detail(c, "emulator-5554", Some("local")).unwrap();
    let location = DeviceAction { device_id: s("emulator-5554"), host_id: None, action: Action::SetLocation { latitude: -33.45, longitude: -70.66 } };
    api::device::action(c, &location).unwrap();
    let push = DeviceAction {
        device_id: s("booted"),
        host_id: Some(s("mac-mini")),
        action: Action::SendPush { app_id: s("com.example"), payload: json!({"aps":{"alert":"hi"}}) },
    };
    api::device::action(c, &push).unwrap();
    // scheduledTasks
    api::scheduled_tasks::list(c).unwrap();
    api::scheduled_tasks::upsert(c, &TaskUpsert::from_json(&task_args()).unwrap()).unwrap();
    let hook = TaskUpsert {
        command_id: None,
        id: Some(s("scheduled-task:hook-1")),
        require_existing: None,
        title: s("Triage"),
        prompt: s("Triage {{body.issue}}"),
        enabled: false,
        schedule: webhook_task_args()["schedule"].clone(),
        project_id: s("p-1"),
        thread_id: Some(None),
        workspace_strategy: WorkspaceStrategy::ExistingWorktree { worktree_path: s("/w/t"), branch: None },
        model_selection: json!({"instanceId":"claude","model":"opus"}),
        runtime_mode: s("approval-required"),
        interaction_mode: s("plan"),
        created_by: None,
        creation_source: None,
    };
    api::scheduled_tasks::upsert(c, &hook).unwrap();
    api::scheduled_tasks::set_enabled(c, "scheduled-task:omarchy-s1", false).unwrap();
    api::scheduled_tasks::delete(c, "scheduled-task:omarchy-s1").unwrap();
    api::scheduled_tasks::run_now(c, "scheduled-task:omarchy-s1").unwrap();
    // small modules
    api::agent_sessions::scan(c).unwrap();
    api::agent_sessions::import(c, "p-1", Some("/r")).unwrap();
    api::mcp_apps::call_tool(c, "t-1", "item-1", "get_weather", &json!({"city":"Santiago"})).unwrap();
    api::mcp_apps::read_resource(c, "t-1", "item-1", "ui://weather/app.html").unwrap();
    api::mcp_apps::update_model_context(c, "t-1", "item-1", "t-1", Some(&[json!({"type":"text","text":"72F"})]), None).unwrap();
    api::shell::open_in_editor(c, "/r", "zed", None).unwrap();
    assert_eq!(format!("{:?}", SecretAnswer::Save(s("hunter2"))), "Save([redacted])");
    api::secrets::answer(c, "t-1", "item-2", &SecretAnswer::Decline).unwrap();
    api::worktree_setup::cancel(c, "t-1").unwrap();
    let terminal = TerminalOpen {
        thread_id: s("t-1"),
        terminal_id: s("term-1"),
        cwd: s("/r"),
        worktree_path: Some(None),
        cols: Some(120),
        rows: Some(40),
        env: json!({"TERM":"xterm-256color"}).as_object().cloned(),
        provider_instance_id: Some(s("codex")),
    };
    api::terminal::open(c, &terminal).unwrap();
    api::terminal::write(c, "t-1", "term-1", " ls\r").unwrap();
    api::terminal::resize(c, "t-1", "term-1", 100, 30).unwrap();
    api::orchestration::get_thread_family(c,"t-1").unwrap();
    api::orchestration::get_worktree_thread_ids(c,"/w/t").unwrap();
    api::orchestration::get_older_settled_threads(c,&json!({"settledAt":"2026-10-26T00:00:00Z"}),None,50).unwrap();
    api::guest_share::create(c,"t-1").unwrap();
    api::projects::list_ignored_files(c,"/r").unwrap();
    api::device::eas_account(c,Some("eas1"),None).unwrap();
    api::pull_requests::change_breakdown(c,&pr).unwrap();
    api::pull_requests::filter_options(c,"labels",Some("p-1"),None,Some("bug")).unwrap();
    let recorded = fake.finish();
    assert_eq!(wire(&recorded), expected(&rows));
}

#[test]
fn local_rejections_never_open_a_socket() {
    let fake = FakeT3::spawn(vec![]);
    let conn = fake.connection();
    let long = "x".repeat(513);
    let cases = [
        ("vcs.refreshStatus", json!({"cwd":""}), "invalid_cwd"),
        ("vcs.refreshStatus", json!({}), "invalid_cwd"),
        ("vcs.refreshStatus", json!({"cwd":"/r","extra":1}), "invalid_input"),
        ("vcs.refreshStatus", json!("not an object"), "invalid_input"),
        ("vcs.listRefs", json!({"cwd":"/r","limit":201}), "invalid_limit"),
        ("vcs.init", json!({"cwd":"/r","kind":"svn"}), "invalid_kind"),
        ("git.preparePullRequestThread", json!({"cwd":"/r","reference":"1","mode":"remote"}), "invalid_mode"),
        ("preview.list", json!({"threadId":"../t"}), "invalid_thread_id"),
        ("preview.adjust", json!({"threadId":"t","tabId":"tab","zoomFactor":1.2}), "invalid_zoom_factor"),
        ("preview.open", json!({"threadId":"t","viewport":{"_tag":"freeform","width":3840,"height":3840}}), "invalid_viewport"),
        ("preview.reportStatus", json!({"threadId":"t","tabId":"tab","navStatus":{"_tag":"Gone"},"canGoBack":true,"canGoForward":true}), "invalid_nav_status"),
        ("orchestration.dispatchCommand", json!({"type":"thread.archive","commandId":"bad id","threadId":"t"}), "invalid_command_id"),
        ("orchestration.dispatchCommand", json!({"type":"thread.archive","threadId":"t"}), "invalid_command_id"),
        ("orchestration.dispatchCommand", json!({"type":"thread.archive","commandId":"c","threadId":"t","x":1}), "invalid_input"),
        ("orchestration.dispatchCommand", json!({"type":"thread.explode","commandId":"c"}), "invalid_type"),
        ("orchestration.launchThread", with(launch_args(), json!({"title":"   "})), "invalid_title"),
        ("orchestration.launchThread", with(launch_args(), json!({"runtimeMode":"yolo"})), "invalid_runtime_mode"),
        ("orchestration.launchThread", with(launch_args(), json!({"workspaceStrategy":{"type":"cloud"}})), "invalid_workspace_strategy"),
        ("orchestration.launchThread", with(launch_args(), json!({"commandId":null})), "invalid_command_id"),
        ("orchestration.getTurnDiff", json!({"threadId":"t","fromTurnCount":4,"toTurnCount":3}), "invalid_from_turn_count"),
        ("orchestration.searchThreads", json!({"query":" a "}), "invalid_query"),
        ("server.updateSettings", json!({"patch":[]}), "invalid_patch"),
        ("server.updateSettings", json!({"patch":{},"providerInstanceMutation":{"operation":"remove","instanceId":"9bad"}}), "invalid_provider_instance_mutation"),
        ("server.reportClientActivity", with(activity_args(), json!({"observedAt":"yesterday"})), "invalid_observed_at"),
        ("server.reportClientActivity", with(activity_args(), json!({"scopes":[{"type":"everything"}]})), "invalid_scopes"),
        ("server.upsertKeybinding", json!({"key":"k","command":"rm -rf"}), "invalid_command"),
        ("server.upsertKeybinding", json!({"key":"k","command":"thread.next","replace":{"key":"k"}}), "invalid_replace"),
        ("server.prepareAcpRegistryAgent", json!({"agentId":"Bad/Agent"}), "invalid_agent_id"),
        ("server.signalProcess", json!({"pid":1,"startTimeMs":0,"signal":"SIGTERM"}), "invalid_signal"),
        ("server.signalProcess", json!({"pid":0,"startTimeMs":0,"signal":"SIGINT"}), "invalid_pid"),
        ("server.checkOtlpEndpoint", json!({"signal":"traces","url":"ftp://x"}), "invalid_url"),
        ("server.getUsageSummary", json!({"sinceDay":"2026-10-24","untilDay":"2026-10-01","timeZone":"UTC"}), "invalid_until_day"),
        ("server.getUsageSummary", json!({"sinceDay":"24/10/2026","untilDay":"2026-10-24","timeZone":"UTC"}), "invalid_since_day"),
        ("server.refreshProviders", json!({"instanceId":"1codex"}), "invalid_instance_id"),
        ("provider.auth.respond", json!({"instanceId":"c","flowId":"f","interactionId":"i","response":{"type":"browser","action":"maybe"}}), "invalid_response"),
        ("provider.auth.respond", json!({"instanceId":"c","flowId":"f","interactionId":"i","response":{"type":"terminal","data":"x","size":{"cols":501,"rows":1}}}), "invalid_response"),
        ("provider.install.cancel", json!({"instanceId":"c","operationId":"x".repeat(129)}), "invalid_operation_id"),
        ("assets.createUrl", json!({"resource":{"_tag":"attachment","attachmentId":" "}}), "invalid_attachment_id"),
        ("assets.createUrl", json!({"resource":{"_tag":"workspace-file","threadId":"t","path":"a","x":1}}), "invalid_input"),
        ("assets.createUrl", json!({"resource":{"_tag":"anything"}}), "invalid_resource"),
        ("attachments.delete", json!({"attachmentId":"a".repeat(257)}), "invalid_attachment_id"),
        ("review.getDiffPreview", json!({"cwd":"/r","file":{"path":"a"}}), "invalid_file"),
        ("review.getDiffPreview", json!({"cwd":"/r","baseRef":"  "}), "invalid_base_ref"),
        ("review.getDiffFileContents", json!({"cwd":"/r","sourceKind":"working-tree","changeType":"moved","oldPath":"a","newPath":"b"}), "invalid_change_type"),
        ("projects.readFile", json!({"cwd":"/r","relativePath":long}), "invalid_relative_path"),
        ("projects.readFile", json!({"cwd":"/r","relativePath":"a","allowMissing":"yes"}), "invalid_allow_missing"),
        ("projects.searchEntries", json!({"cwd":"/r","query":"","limit":0}), "invalid_limit"),
        ("projects.mutate", json!({"type":"project.rename","commandId":"c","projectId":"p"}), "invalid_type"),
        ("projects.mutate", json!({"type":"project.delete","projectId":"p"}), "invalid_command_id"),
        ("projects.mutate", with(mutate_update_args(), json!({"projectIcon":{"kind":"lucide","name":"Bad Name","color":"red"}})), "invalid_project_icon"),
        ("projects.mutate", with(mutate_update_args(), json!({"scripts":[{"id":"b","name":"B","command":"x","icon":"rocket","runOnWorktreeCreate":true}]})), "invalid_scripts"),
        ("projects.mutate", json!({"type":"project.create","commandId":"c","projectId":"p","title":"T","workspaceRoot":"/r","autoPull":true}), "invalid_input"),
        ("projects.createNew", json!({"name":"n".repeat(201)}), "invalid_name"),
        ("projectClone.start", json!({"projectId":"p","title":"T","createdAt":"now","destinationPath":"/d"}), "invalid_created_at"),
        ("projectClone.start", json!({"projectId":"p","title":"T","createdAt":"2026-10-24T10:00:00Z","destinationPath":"/d","provider":"GitHub"}), "invalid_provider"),
        ("filesystem.browse", json!({"partialPath":long}), "invalid_partial_path"),
        ("filesystem.getMetadata", json!({"paths":[]}), "invalid_paths"),
        ("filesystem.getMetadata", json!({"paths":[1]}), "invalid_paths"),
        ("pullRequests.detail", with(pr(), json!({"number":0})), "invalid_number"),
        ("pullRequests.list", json!({"state":"draft"}), "invalid_state"),
        ("pullRequests.list", json!({"state":"open","filters":{"draft":"only","stars":5}}), "invalid_filters"),
        ("pullRequests.comment", with(pr(), json!({"body":""})), "invalid_body"),
        ("pullRequests.runAction", with(pr(), json!({"action":"merge","updateMethod":"squash"})), "invalid_update_method"),
        ("pullRequests.submitReview", with(pr(), json!({"verdict":"approve","body":"","comments":[{"path":"a","body":"b","position":{"kind":"added","newLine":0}}]})), "invalid_comments"),
        ("pullRequests.requestReviewers", with(pr(), json!({"reviewers":[],"requested":true})), "invalid_reviewers"),
        ("pullRequests.setLabels", with(pr(), json!({"labels":["bug"]})), "invalid_applied"),
        ("device.list", json!({"updateTool":"all"}), "invalid_update_tool"),
        ("device.testHost", json!({"id":"local","label":"L","target":"me@h"}), "invalid_id"),
        ("device.testHost", json!({"id":"h","label":"L","target":"-oProxyCommand=x"}), "invalid_target"),
        ("device.testHost", json!({"id":"h","label":"L","target":"me@h","port":70000}), "invalid_port"),
        ("device.open", json!({"threadId":"t","deviceId":"d","platform":"windows"}), "invalid_platform"),
        ("device.action", json!({"type":"setAppearance","deviceId":"d","value":"blue"}), "invalid_value"),
        ("device.action", json!({"type":"setLocation","deviceId":"d","latitude":91,"longitude":0}), "invalid_latitude"),
        ("device.action", json!({"type":"selfDestruct","deviceId":"d"}), "invalid_type"),
        ("device.action", json!({"type":"shake","deviceId":"d","value":"x"}), "invalid_input"),
        ("scheduledTasks.upsert", with(task_args(), json!({"commandId":null})), "invalid_command_id"),
        ("scheduledTasks.upsert", with(task_args(), json!({"schedule":{"type":"interval","everyMs":59999}})), "invalid_schedule"),
        ("scheduledTasks.upsert", with(task_args(), json!({"schedule":{"type":"fixed_time","timeOfDay":"24:00"}})), "invalid_schedule"),
        ("scheduledTasks.upsert", with(task_args(), json!({"interactionMode":"chat"})), "invalid_interaction_mode"),
        ("scheduledTasks.runNow", json!({"id":""}), "invalid_id"),
        ("agentSessions.import", json!({"projectId":"p","expectedWorkspaceRoot":"\u{7}"}), "invalid_expected_workspace_root"),
        ("mcpApps.callTool", json!({"threadId":"t","itemId":"i","name":"n","arguments":[]}), "invalid_arguments"),
        ("mcpApps.updateModelContext", json!({"threadId":"t","itemId":"i","conversationThreadId":"bad id"}), "invalid_conversation_thread_id"),
        ("shell.openInEditor", json!({"cwd":"/r","editor":"notepad"}), "invalid_editor"),
        ("secrets.answerRequest", json!({"threadId":"t","turnItemId":"i","answer":{"type":"save","secret":"  "}}), "invalid_answer"),
        ("worktreeSetup.cancel", json!({"threadId":""}), "invalid_thread_id"),
        ("terminal.open", json!({"threadId":"t","terminalId":"term-1","cwd":"/r","env":{"1BAD":"x"}}), "invalid_env"),
        ("terminal.write", json!({"threadId":"t","terminalId":"term-1","data":""}), "invalid_data"),
        ("terminal.resize", json!({"threadId":"t","terminalId":"term-1","cols":1001,"rows":30}), "invalid_cols"),
        ("no.such.function", json!({}), "unknown_function"),
    ];
    for (name, args, error) in cases {
        assert_eq!(api::call(&conn, name, &args, None).unwrap_err(), error, "{name} {args}");
    }
    assert_eq!(api::vcs::refresh_status(&conn, "\u{7}").unwrap_err(), "invalid_cwd");
    assert_eq!(api::preview::list(&conn, "").unwrap_err(), "invalid_thread_id");
    assert_eq!(api::projects::read_file(&conn, "/r", "", None).unwrap_err(), "invalid_relative_path");
    assert_eq!(api::filesystem::metadata(&conn, &vec!["/a".to_string(); 65]).unwrap_err(), "invalid_paths");
    assert_eq!(api::terminal::resize(&conn, "t", "term-1", 80, 0).unwrap_err(), "invalid_rows");
    let recorded = fake.finish();
    assert_eq!(recorded.connections, 0, "validation happens before any socket");
}

#[test]
fn every_module_has_its_unary_functions_and_scopes() {
    let mut counts = std::collections::BTreeMap::new();
    for f in api::functions().filter(|f| !f.stream && f.tag.is_some()) {
        *counts.entry(f.module).or_insert(0) += 1;
    }
    let expected = [
        ("agentSessions", 2), ("assets", 2), ("device", 8), ("diagnostics", 8), ("filesystem", 2), ("guestShare",1), ("mcpApps", 3),
        ("orchestration", 13), ("preview", 9), ("projectClone", 1), ("projects", 5), ("provider", 9), ("pullRequests", 15),
        ("review", 2), ("scheduledTasks", 5), ("secrets", 1), ("server", 9), ("shell", 1), ("terminal", 3), ("vcs", 7),
        ("worktreeSetup", 1),
    ];
    assert_eq!(counts.into_iter().collect::<Vec<_>>(), expected);
    assert_eq!(api::functions().filter(|f| !f.stream && f.tag.is_some()).count(), 107);
    // Payload-dependent scopes the host can compute before choosing a token.
    assert_eq!(api::device::list_scope(Some("hub"), Some(true), None), "orchestration:operate");
    assert_eq!(api::device::list_scope(None, Some(true), Some("mac")), "orchestration:read");
    assert_eq!(api::device::list_scope(None, None, Some("mac")), "orchestration:operate");
    assert_eq!(api::vcs::prepare_pull_request_thread_scopes("local", Some("t")), ["source-control:write"]);
    let delete = ProjectMutation::from_command("omarchy-p", &api::projects::command(&json!({"action":"project-remove","projectId":"p-1","confirmed":true})).unwrap()).unwrap();
    assert_eq!(delete.required_scopes(), ["orchestration:operate"]);
    assert_eq!(delete.payload().unwrap(), json!({"type":"project.delete","commandId":"omarchy-p","projectId":"p-1","force":false}));
    let scripts = ProjectMutation::from_json(&mutate_update_args()).unwrap();
    assert_eq!(scripts.required_scopes(), ["orchestration:operate", "settings:write"]);
}
