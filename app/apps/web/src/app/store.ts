import { create } from "zustand";
import type { Notification, User } from "@/shared/types/domain";

interface AppState {
  user: User | null;
  token: string | null;
  notifications: Notification[];
  setSession: (user: User | null, token: string | null) => void;
  setNotifications: (notifications: Notification[]) => void;
}

export const useAppStore = create<AppState>((set) => ({
  user: null,
  token: null,
  notifications: [],
  setSession: (user, token) => set({ user, token }),
  setNotifications: (notifications) => set({ notifications })
}));
