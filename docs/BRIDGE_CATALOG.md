# Client-scoped bridge catalog contract (proposal, not a deployed service)

The desktop catalog is not an authorization grant to a phone. This module implements
credential-free export and reconciliation; it does not add a listener, device enrollment,
T3 token delegation, network discovery, or Android integration by itself.

## Trusted boundary

The publisher authenticates the paired bridge client and resolves its **explicit grant**
to allowed environment UUIDs. It obtains the sanitized `connection_catalog::view` and
checks every portable route's public T3 descriptor against that UUID. It then invokes
`connectionCatalog.publishBridge(view, authorized, endpoints, context)`. `authorized`
is internal trusted state, never a client-supplied list. Do not pass the encrypted
native catalog or its credentials to a client.

Endpoints are objects `{environmentId, descriptorEnvironmentId, id, url, priority}`.
Only HTTPS origins without userinfo/query/fragment/path are portable. Literal loopback,
unspecified, multicast and link-local addresses (including mapped IPv4) are rejected.
This validation does not prove network reachability, resolve DNS or replace the normal
TLS/descriptor identity check at connection time. A desktop SSH forward is never exported.

Context: `{bridgeId, clientId, revision, now}`. `now` is epoch milliseconds; revision
is a monotonically increasing integer **per bridge/client grant**, advanced whenever
the authorized view, name or routes changes. A changed grant must advance revision too.
Output expires after ten minutes and contains only:

```json
{"version":1,"bridgeId":"example-bridge","clientId":"example-client","revision":1,
 "issuedAt":1000,"expiresAt":601000,"environments":[
   {"environmentId":"example-node-id","label":"NODE","routes":[
     {"id":"public-gateway","url":"https://node.example.net/","priority":0}]}]}
```

A trusted transport must authenticate publisher and recipient before invoking
`connectionCatalog.reconcileBridge(snapshot, cache, context)`. Initial domain cache
is `{}` (Rust API accepts null). Reconcile context is `{bridgeId, clientId, now}`.
Cross-identity input, expired snapshots, excessive future timestamps, revision rollback, changed equal revisions,
duplicate identities/routes, excessive sizes, and unrecognized fields are rejected.
Error codes never echo document contents or credentials.

Catalog admission tolerates `issuedAt` up to 30,000 ms ahead of the receiver's
clock, inclusive. This fixed allowance handles small clock differences between
devices; callers cannot override it. It does not rewrite timestamps or add grace
to `expiresAt`: a snapshot is rejected exactly at its absolute expiration.
The declared `expiresAt - issuedAt` window must remain positive and at most
600,000 ms. A receiver behind the publisher can therefore observe up to that
window plus the clock allowance before the original absolute deadline.

After identity validation, invalid windows return `bridge_catalog_invalid_window`,
oversized windows return `bridge_catalog_ttl_exceeded`, issue timestamps more than
30,000 ms ahead return `bridge_catalog_from_future`, and actual expiration returns
`bridge_catalog_expired`, in that order. Revision and route guards remain unchanged.
Discovery is credential-free; this allowance never extends enrollment or T3 session
authorization and must not be applied to their independent lifetime checks.

## Persistence and authorization

The returned cache is only this publisher/client's slice. Removed environments become
`removed:true` tombstones with no routes. Preserve local entries, stable local IDs,
drafts and reading anchors. Never treat network failure as an empty snapshot. Keep
cached rows visible as offline; cached discovery must not restore a revoked grant.
The host must stop affected connections on an authenticated grant withdrawal; transport
errors alone do not prove withdrawal. Cache exhaustion returns an error rather than
silently evicting identities; an explicit host retention policy is still needed.

A manifest contains no bearer, pairing code, private key or authorization to operate
T3. A separately authenticated enrollment/delegation step must supply each client's
bounded T3 session (or equivalent capability with enforced environment/scope/lifetime).
Existing desktop tokens must never be copied. The current module intentionally cannot
manufacture that authorization. Re-authentication/revocation and a remote transport
are required before a phone can automatically show authorized hosts' threads.

## Validation

Rust tests exercise VQ+NODE, explicit grant filtering, descriptor mismatch, route change,
withdrawal/recovery, idempotence, rollback, cross-client binding, expiry, secret fields,
loopback rejection, malformed input and cache preservation. These are isolated tests;
they do not claim a deployed bridge, physical Android sync or live thread access.
