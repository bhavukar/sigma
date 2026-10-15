export interface Env {
  SIGMA_ROOM: DurableObjectNamespace;
  ASSETS?: Fetcher;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    // WebSocket route for real-time multiplayer
    if (url.pathname === "/ws" || url.pathname.startsWith("/ws/")) {
      const roomId = url.searchParams.get("room") || "default";
      const id = env.SIGMA_ROOM.idFromName(roomId);
      const room = env.SIGMA_ROOM.get(id);
      return room.fetch(request);
    }

    // Health check endpoint
    if (url.pathname === "/health") {
      return new Response(JSON.stringify({ status: "ok", service: "sigma-sync" }), {
        headers: { "Content-Type": "application/json" },
      });
    }

    // Serve static assets if available (Web app)
    if (env.ASSETS) {
      return env.ASSETS.fetch(request);
    }

    return new Response("Sigma Real-time Multiplayer Engine (Cloudflare Workers)", {
      status: 200,
    });
  },
};

export class SigmaRoom implements DurableObject {
  state: DurableObjectState;
  sessions: Map<WebSocket, { userId: string; userName: string; color: string }>;
  latestDoc: any = null;

  constructor(state: DurableObjectState, env: Env) {
    this.state = state;
    this.sessions = new Map();
  }

  async fetch(request: Request): Promise<Response> {
    if (request.headers.get("Upgrade") !== "websocket") {
      return new Response("Expected websocket", { status: 426 });
    }

    const pair = new WebSocketPair();
    const [client, server] = Object.values(pair);

    server.accept();

    server.addEventListener("message", (event) => {
      try {
        const msg = JSON.parse(event.data as string);

        if (msg.type === "join") {
          this.sessions.set(server, {
            userId: msg.user_id,
            userName: msg.user_name,
            color: msg.color,
          });

          // Send current peers and latest document snapshot
          const peers = Array.from(this.sessions.values()).map((s) => ({
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

      if (session) {
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
