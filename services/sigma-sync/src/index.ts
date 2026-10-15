export interface Env {
  SIGMA_ROOM: DurableObjectNamespace;
  ASSETS?: Fetcher;
}

const SECURITY_HEADERS: Record<string, string> = {
  "X-Content-Type-Options": "nosniff",
  "X-Frame-Options": "SAMEORIGIN",
  "Referrer-Policy": "strict-origin-when-cross-origin",
  "Access-Control-Allow-Origin": "*",
};

// Security constants
const MAX_ROOM_ID_LEN = 64;
const MAX_PAYLOAD_BYTES = 512 * 1024; // 512 KB
const MAX_SESSIONS_PER_ROOM = 50;
const MAX_MESSAGES_PER_SEC = 120;

function sanitizeString(str: unknown, maxLen: number): string {
  if (typeof str !== "string") return "";
  return str.slice(0, maxLen).replace(/[<>]/g, "");
}

function sanitizeRoomId(roomId: string | null): string {
  if (!roomId) return "default";
  const cleaned = roomId.trim().slice(0, MAX_ROOM_ID_LEN).replace(/[^a-zA-Z0-9_-]/g, "");
  return cleaned || "default";
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    // WebSocket route for real-time multiplayer
    if (url.pathname === "/ws" || url.pathname.startsWith("/ws/")) {
      const rawRoom = url.searchParams.get("room");
      const roomId = sanitizeRoomId(rawRoom);
      const id = env.SIGMA_ROOM.idFromName(roomId);
      const room = env.SIGMA_ROOM.get(id);
      return room.fetch(request);
    }

    // Better Auth API routes
    if (url.pathname.startsWith("/api/auth/")) {
      const authPath = url.pathname.replace("/api/auth", "");
      const corsHeaders = {
        "Content-Type": "application/json",
        "Access-Control-Allow-Origin": "*",
        "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
        "Access-Control-Allow-Headers": "Content-Type, Authorization",
      };

      if (request.method === "OPTIONS") {
        return new Response(null, { status: 204, headers: corsHeaders });
      }

      if (authPath === "/session" || authPath === "/get-session") {
        const authHeader = request.headers.get("Authorization");
        if (authHeader && authHeader.startsWith("Bearer ")) {
          const token = authHeader.slice(7);
          return new Response(
            JSON.stringify({
              session: {
                id: `sess_${token.slice(0, 8)}`,
                userId: "usr_designer",
                token,
                expiresAt: new Date(Date.now() + 7 * 86400000).toISOString(),
              },
              user: {
                id: "usr_designer",
                name: "Pro Designer",
                email: "designer@sigma.design",
                role: "Pro Designer",
                emailVerified: true,
              },
            }),
            { headers: corsHeaders }
          );
        }
        return new Response(JSON.stringify({ session: null, user: null }), { headers: corsHeaders });
      }

      if (authPath === "/sign-in/email" || authPath === "/sign-up/email") {
        try {
          const body: any = await request.json();
          const email = sanitizeString(body?.email, 128) || "designer@sigma.design";
          const name = sanitizeString(body?.name, 64) || email.split("@")[0] || "Designer";
          const role = sanitizeString(body?.role, 32) || "Pro Designer";
          const token = `ba_tok_${Math.random().toString(36).slice(2)}_${Date.now()}`;
          const userId = `usr_${Math.random().toString(36).slice(2, 8)}`;

          return new Response(
            JSON.stringify({
              session: {
                id: `sess_${Date.now()}`,
                userId,
                token,
                expiresAt: new Date(Date.now() + 14 * 86400000).toISOString(),
              },
              user: {
                id: userId,
                name,
                email,
                role,
                image: `https://api.dicebear.com/7.x/initials/svg?seed=${encodeURIComponent(name)}`,
                emailVerified: true,
              },
            }),
            { headers: corsHeaders }
          );
        } catch {
          return new Response(JSON.stringify({ error: "Invalid JSON payload" }), {
            status: 400,
            headers: corsHeaders,
          });
        }
      }

      if (authPath === "/sign-out") {
        return new Response(JSON.stringify({ success: true }), { headers: corsHeaders });
      }

      return new Response(JSON.stringify({ status: "ok", provider: "better-auth" }), {
        headers: corsHeaders,
      });
    }

    // Health check endpoint
    if (url.pathname === "/health") {
      return new Response(JSON.stringify({ status: "ok", service: "sigma-sync", v: "2.1.0", auth: "better-auth" }), {
        headers: {
          "Content-Type": "application/json",
          ...SECURITY_HEADERS,
        },
      });
    }

    // Serve static assets if available (Web app)
    if (env.ASSETS) {
      const assetResp = await env.ASSETS.fetch(request);
      const newHeaders = new Headers(assetResp.headers);
      for (const [k, v] of Object.entries(SECURITY_HEADERS)) {
        newHeaders.set(k, v);
      }
      return new Response(assetResp.body, {
        status: assetResp.status,
        statusText: assetResp.statusText,
        headers: newHeaders,
      });
    }

    return new Response("Sigma Real-time Multiplayer Engine (Cloudflare Workers)", {
      status: 200,
      headers: SECURITY_HEADERS,
    });
  },
};

interface SessionMeta {
  userId: string;
  userName: string;
  color: any;
  msgCount: number;
  lastResetTime: number;
}

export class SigmaRoom implements DurableObject {
  state: DurableObjectState;
  sessions: Map<WebSocket, SessionMeta>;
  latestDoc: any = null;
  loaded: boolean = false;

  constructor(state: DurableObjectState, env: Env) {
    this.state = state;
    this.sessions = new Map();
  }

  async ensureLoaded() {
    if (!this.loaded) {
      try {
        this.latestDoc = (await this.state.storage.get("latestDoc")) || null;
      } catch (e) {
        console.error("Failed to load initial doc from SQLite storage:", e);
      }
      this.loaded = true;
    }
  }

  async fetch(request: Request): Promise<Response> {
    if (request.headers.get("Upgrade") !== "websocket") {
      return new Response("Expected websocket", { status: 426, headers: SECURITY_HEADERS });
    }

    await this.ensureLoaded();

    if (this.sessions.size >= MAX_SESSIONS_PER_ROOM) {
      return new Response("Room capacity reached", { status: 429, headers: SECURITY_HEADERS });
    }

    const pair = new WebSocketPair();
    const [client, server] = Object.values(pair);

    server.accept();

    const sessionMeta: SessionMeta = {
      userId: "",
      userName: "Designer",
      color: { r: 10, g: 132, b: 255, a: 255 },
      msgCount: 0,
      lastResetTime: Date.now(),
    };
    this.sessions.set(server, sessionMeta);

    server.addEventListener("message", async (event) => {
      try {
        if (typeof event.data !== "string") {
          return;
        }

        // Payload size protection
        if (event.data.length > MAX_PAYLOAD_BYTES) {
          server.close(1009, "Message payload too large");
          return;
        }

        // Rate limiting sliding window
        const now = Date.now();
        if (now - sessionMeta.lastResetTime > 1000) {
          sessionMeta.msgCount = 0;
          sessionMeta.lastResetTime = now;
        }
        sessionMeta.msgCount++;
        if (sessionMeta.msgCount > MAX_MESSAGES_PER_SEC) {
          // Drop excessive flood messages to protect peers
          return;
        }

        const msg = JSON.parse(event.data);
        if (!msg || typeof msg !== "object" || !msg.type) {
          return;
        }

        if (msg.type === "join") {
          sessionMeta.userId = sanitizeString(msg.user_id, 48) || `user_${Date.now()}`;
          sessionMeta.userName = sanitizeString(msg.user_name, 32) || "Designer";
          if (msg.color && typeof msg.color === "object") {
            sessionMeta.color = msg.color;
          }

          // Send current peers and latest document snapshot
          const peers = Array.from(this.sessions.values())
            .filter((s) => s.userId.length > 0)
            .map((s) => ({
              id: s.userId,
              name: s.userName,
              color: s.color,
            }));

          server.send(
            JSON.stringify({
              type: "room_state",
              peers,
              doc: this.latestDoc,
            })
          );
        }

        if (msg.type === "doc_update" && msg.doc) {
          this.latestDoc = msg.doc;
          // Asynchronously persist to SQLite storage without blocking WebSocket dispatch
          this.state.storage.put("latestDoc", this.latestDoc).catch((err) => {
            console.error("Storage persistence error:", err);
          });
        }

        // Broadcast to all other connected peers in this room
        for (const [ws] of this.sessions) {
          if (ws !== server && ws.readyState === WebSocket.OPEN) {
            ws.send(event.data);
          }
        }
      } catch (err) {
        console.error("Error processing websocket message:", err);
      }
    });

    const closeHandler = () => {
      const session = this.sessions.get(server);
      this.sessions.delete(server);

      if (session && session.userId) {
        const leaveMsg = JSON.stringify({
          type: "leave",
          user_id: session.userId,
        });
        for (const [ws] of this.sessions) {
          if (ws.readyState === WebSocket.OPEN) {
            ws.send(leaveMsg);
          }
        }
      }
    };

    server.addEventListener("close", closeHandler);
    server.addEventListener("error", closeHandler);

    return new Response(null, { status: 101, webSocket: client });
  }
}
