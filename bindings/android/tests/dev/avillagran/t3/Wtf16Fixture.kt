package dev.avillagran.t3
/** Runs only through android-sandbox against the private admitted native JS peer. */
fun main(args: Array<String>) {
    check(args.size == 1)
    T3Client().use { client ->
        client.connectBearer(args[0], "synthetic-wtf16")
        val queries = listOf("ab\uD800", "ab\uDC00", "ab\uD83D\uDE00", "a".repeat(199) + '\uD83D')
        for (query in queries) {
            val encoded = client.utf16JsonString(query).toString(Charsets.US_ASCII)
            check(encoded.contains("\\u"))
            val raw = client.searchThreadsUtf16(query).toString(Charsets.UTF_8)
            check(raw.contains("\"ok\":true"))
            check(raw.contains("\\ud83d")) // The actual native snippet ends at a high unit.
            if (query.endsWith('\uD800')) check(raw.contains("ab\\ud800"))
            if (query.endsWith('\uDC00')) check(raw.contains("ab\\udc00"))
        }
        val invalid = client.executeRaw("{\"op\":\"rpc\",\"tag\":\"x\",\"payload\":\"\\uxxxx\"}".toByteArray(Charsets.US_ASCII))
        check(invalid.toString(Charsets.UTF_8).contains("malformed"))
    }
    println("WTF16_JNI_KOTLIN_PASS high low pair clipped nativeSnippet malformed")
}
