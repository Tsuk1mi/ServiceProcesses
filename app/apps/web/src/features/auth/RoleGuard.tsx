import { Navigate, Outlet } from "react-router-dom";
import { useAppStore } from "@/app/store";
import type { Role } from "@/shared/types/domain";

export function RoleGuard({ roles }: { roles: Role[] }) {
  const role = useAppStore((state) => state.user?.role);

  if (!role || !roles.includes(role)) {
    return <Navigate to="/dashboard" replace />;
  }

  return <Outlet />;
}
