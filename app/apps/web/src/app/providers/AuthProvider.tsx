import { createContext, useContext, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { serviceDeskApi } from "@/shared/api/serviceDeskApi";
import type { User } from "@/shared/types/domain";
import { useAppStore } from "@/app/store";

interface AuthContextValue {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  login: (username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
}

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const { user, token, setSession } = useAppStore();
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    const restoreSession = async () => {
      const raw = localStorage.getItem("service-desk-session");
      if (!raw) {
        setLoaded(true);
        return;
      }

      try {
        const session = JSON.parse(raw) as { user: User; token: string };
        setSession(session.user, session.token);
        const actualUser = await serviceDeskApi.auth.me();
        setSession(actualUser, session.token);
        localStorage.setItem("service-desk-session", JSON.stringify({ user: actualUser, token: session.token }));
      } catch {
        setSession(null, null);
        localStorage.removeItem("service-desk-session");
      } finally {
        setLoaded(true);
      }
    };

    void restoreSession();
  }, [setSession]);

  const value = useMemo<AuthContextValue>(
    () => ({
      user,
      token,
      isAuthenticated: Boolean(user && token),
      login: async (username, password) => {
        const response = await serviceDeskApi.auth.login({ username, password });
        setSession(response.user, response.accessToken);
        localStorage.setItem("service-desk-session", JSON.stringify({ user: response.user, token: response.accessToken }));
      },
      logout: async () => {
        try {
          await serviceDeskApi.auth.logout();
        } finally {
          setSession(null, null);
          localStorage.removeItem("service-desk-session");
        }
      }
    }),
    [setSession, token, user]
  );

  if (!loaded) {
    return null;
  }

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error("useAuth must be used inside AuthProvider");
  }
  return context;
}
