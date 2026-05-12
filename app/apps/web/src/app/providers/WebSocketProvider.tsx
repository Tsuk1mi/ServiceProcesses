import { createContext, useContext, useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import { useAppStore } from "@/app/store";

interface WebSocketContextValue {
  connected: boolean;
}

const WebSocketContext = createContext<WebSocketContextValue>({ connected: false });

export function WebSocketProvider({ children }: { children: ReactNode }) {
  const token = useAppStore((state) => state.token);
  const setNotifications = useAppStore((state) => state.setNotifications);
  const socketRef = useRef<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    serviceDeskApi.notifications.list().then(setNotifications);

    const wsUrl = import.meta.env.VITE_WS_URL;
    if (!token || !wsUrl) {
      return undefined;
    }

    const socket = new WebSocket(`${wsUrl}?token=${encodeURIComponent(token)}`);
    socketRef.current = socket;
    socket.onopen = () => setConnected(true);
    socket.onclose = () => setConnected(false);

    socket.onmessage = (event) => {
      const payload = JSON.parse(event.data);
      if (payload.type === "notification") {
        serviceDeskApi.notifications.list().then(setNotifications);
      }
    };

    return () => socket.close();
  }, [setNotifications, token]);

  const value = useMemo(() => ({ connected }), [connected]);

  return <WebSocketContext.Provider value={value}>{children}</WebSocketContext.Provider>;
}

export const useWebSocketStatus = () => useContext(WebSocketContext);
