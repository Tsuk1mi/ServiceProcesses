import {
  BarChart3,
  Bell,
  ClipboardList,
  Gauge,
  Home,
  LogOut,
  Moon,
  Settings,
  ShieldAlert,
  Sun,
  Users,
  Wrench
} from "lucide-react";
import { NavLink, Outlet } from "react-router-dom";
import { useAuth } from "@/app/providers/AuthProvider";
import { useTheme } from "@/app/providers/ThemeProvider";
import { useAppStore } from "@/app/store";
import type { Role } from "@/shared/types/domain";
import { canUseClientWorkspace, isAdminRole, isClientRole, isEmployeeRole } from "@/shared/utils/access";

const navItems = [
  { to: "/dashboard", label: "Кабинет", icon: Home, visible: (role?: Role | null) => Boolean(role) },
  { to: "/requests", label: "Заявки", icon: ClipboardList, visible: (role?: Role | null) => Boolean(role) },
  { to: "/requests/new", label: "Создать заявку", icon: ClipboardList, visible: (role?: Role | null) => canUseClientWorkspace(role) },
  { to: "/work-orders", label: "Наряды", icon: Wrench, visible: (role?: Role | null) => isEmployeeRole(role) },
  { to: "/objects", label: "Объекты", icon: Gauge, visible: (role?: Role | null) => Boolean(role) },
  { to: "/sla/dashboard", label: "SLA", icon: ShieldAlert, visible: (role?: Role | null) => isEmployeeRole(role) },
  { to: "/escalations", label: "Эскалации", icon: Bell, visible: (role?: Role | null) => isEmployeeRole(role) },
  { to: "/analytics/overview", label: "Аналитика", icon: BarChart3, visible: (role?: Role | null) => isEmployeeRole(role) },
  { to: "/users", label: "Пользователи", icon: Users, visible: (role?: Role | null) => isAdminRole(role) },
  { to: "/settings/profile", label: "Настройки", icon: Settings, visible: (role?: Role | null) => Boolean(role) }
];

export function AppLayout() {
  const { user, logout } = useAuth();
  const { theme, toggleTheme } = useTheme();
  const notifications = useAppStore((state) => state.notifications);
  const workspaceTitle = isClientRole(user?.role) ? "Client Portal" : "Employee Workspace";
  const filteredNavItems = navItems.filter((item) => item.visible(user?.role));

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark">SD</span>
          <div>
            <strong>ServiceDesk</strong>
            <small>{workspaceTitle}</small>
          </div>
        </div>
        <nav>
          {filteredNavItems.map((item) => {
            const Icon = item.icon;
            return (
              <NavLink key={item.to} to={item.to}>
                <Icon size={18} />
                <span>{item.label}</span>
              </NavLink>
            );
          })}
        </nav>
      </aside>

      <div className="workspace">
        <header className="topbar">
          <div className="user-chip">
            <strong>{user?.name ?? "Гость"}</strong>
            <small>{user?.role ?? "UNAUTHORIZED"}</small>
          </div>
          <button className="icon-button" onClick={toggleTheme} title="Тема">
            {theme === "light" ? <Moon size={18} /> : <Sun size={18} />}
          </button>
          <NavLink className="notification-button" to="/notifications">
            <Bell size={18} />
            {notifications.filter((item) => !item.read).length ? <span>{notifications.filter((item) => !item.read).length}</span> : null}
          </NavLink>
          <button className="icon-button" onClick={logout} title="Выйти">
            <LogOut size={18} />
          </button>
        </header>
        <main className="content">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
