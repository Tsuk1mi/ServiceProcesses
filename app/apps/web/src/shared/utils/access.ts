import type { Role } from "@/shared/types/domain";

export function isClientRole(role?: Role | null) {
  return role === "CLIENT";
}

export function isEmployeeRole(role?: Role | null) {
  return role === "ADMIN" || role === "MANAGER" || role === "DISPATCHER" || role === "TECHNICIAN";
}

export function canUseClientWorkspace(role?: Role | null) {
  return role === "CLIENT" || role === "ADMIN";
}

export function canUseEmployeeWorkspace(role?: Role | null) {
  return isEmployeeRole(role);
}

export function isAdminRole(role?: Role | null) {
  return role === "ADMIN";
}
