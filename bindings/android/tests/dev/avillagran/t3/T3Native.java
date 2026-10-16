package dev.avillagran.t3;
import java.nio.charset.StandardCharsets;
import java.net.InetSocketAddress;
import com.sun.net.httpserver.HttpServer;
/** Offline host JNI fixture: same native method names/signatures as Kotlin. */
public final class T3Native {
    static { System.loadLibrary("t3_jni"); }
    static native long create();
    static native void free(long handle);
    static native byte[] call(long handle, byte[] request);
    static native byte[] upload(long handle, byte[] metadata, int fd);
    static native byte[][] image(long handle, byte[] metadata);
    static String invoke(long handle, String request) { return new String(call(handle, request.getBytes(StandardCharsets.UTF_8)), StandardCharsets.UTF_8); }
    public static void main(String[] args) throws Exception {
        HttpServer server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
        server.createContext("/fixture", exchange -> {
            if (!"Bearer fixture-token".equals(exchange.getRequestHeaders().getFirst("Authorization"))) throw new AssertionError("missing auth");
            byte[] body = "{\"text\":\"á 🦀\"}".getBytes(StandardCharsets.UTF_8);
            exchange.getResponseHeaders().set("Content-Type", "application/json");
            exchange.sendResponseHeaders(200, body.length); exchange.getResponseBody().write(body); exchange.close();
        });
        server.start();
        long handle = create();
        try {
            if (!invoke(handle, "{\"op\":\"request\",\"method\":\"GET\",\"path\":\"/fixture\"}").contains("not_connected")) throw new AssertionError();
            if (!invoke(handle, "{\"op\":\"connectBearer\",\"httpBaseUrl\":\"http://127.0.0.1:" + server.getAddress().getPort() + "\",\"token\":\"fixture-token\"}").contains("\"ok\":true")) throw new AssertionError();
            if (!invoke(handle, "{\"op\":\"request\",\"method\":\"GET\",\"path\":\"/fixture\"}").contains("á 🦀")) throw new AssertionError("UTF-8 changed");
            if (!invoke(handle, "{").contains("malformed")) throw new AssertionError();
            byte[][] result = image(handle, "{\"id\":\"fixture\",\"name\":\"x.svg\",\"mimeType\":\"image/svg+xml\"}".getBytes(StandardCharsets.UTF_8));
            if (!new String(result[0], StandardCharsets.UTF_8).contains("invalid_image") || result[1] != null) throw new AssertionError();
            try { upload(handle, "{}".getBytes(StandardCharsets.UTF_8), -1); throw new AssertionError(); }
            catch (IllegalStateException expected) { if (!"host_file_unavailable".equals(expected.getMessage())) throw expected; }
            for (int n = 0; n < 200; n++) if (!invoke(handle, "{\"op\":\"unknown\"}").contains("unknown_operation")) throw new AssertionError();
            System.out.println("JNI_PASS: real native calls, Unicode HTTP, image errors, FD rejection and repeated response ownership");
        } finally { free(handle); server.stop(0); }
    }
}
