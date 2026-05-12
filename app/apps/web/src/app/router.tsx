import { Navigate, createBrowserRouter } from "react-router-dom";
import { AppLayout } from "@/app/layout/AppLayout";
import { AuthGuard } from "@/features/auth/AuthGuard";
import { RoleGuard } from "@/features/auth/RoleGuard";
import { ForgotPasswordPage, LoginPage, MfaPage, RegisterPage, ResetPasswordPage } from "@/features/auth/pages";
import { ClientDashboardPage } from "@/features/dashboard/ClientDashboardPage";
import { DashboardPage } from "@/features/dashboard/DashboardPage";
import { RequestsPage } from "@/features/requests/RequestsPage";
import { RequestDetailPage } from "@/features/requests/RequestDetailPage";
import { RequestFormPage } from "@/features/requests/RequestFormPage";
import { RequestsKanbanPage } from "@/features/requests/KanbanPage";
import { WorkOrderDetailPage } from "@/features/work-orders/WorkOrderDetailPage";
import { WorkOrdersPage } from "@/features/work-orders/WorkOrdersPage";
import { ObjectCreatePage } from "@/features/objects/ObjectCreatePage";
import { ObjectDetailPage } from "@/features/objects/ObjectDetailPage";
import { ObjectsPage } from "@/features/objects/ObjectsPage";
import { SlaDashboardPage } from "@/features/sla/SlaDashboardPage";
import { SlaPoliciesPage } from "@/features/sla/SlaPoliciesPage";
import { EscalationRulesPage } from "@/features/escalations/EscalationRulesPage";
import { EscalationsPage } from "@/features/escalations/EscalationsPage";
import { AnalyticsPage } from "@/features/analytics/AnalyticsPage";
import { UsersPage } from "@/features/users/UsersPage";
import { RolesPage } from "@/features/users/RolesPage";
import { NotificationsPage } from "@/features/notifications/NotificationsPage";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { useAppStore } from "@/app/store";
import { isClientRole } from "@/shared/utils/access";

function SimplePage({ title }: { title: string }) {
  return (
    <section className="panel">
      <h1>{title}</h1>
      <p>Экран подключен к маршрутизации и готов для детализации бизнес-логики.</p>
    </section>
  );
}

function DashboardEntryPage() {
  const role = useAppStore((state) => state.user?.role);
  return isClientRole(role) ? <ClientDashboardPage /> : <DashboardPage />;
}

export const router = createBrowserRouter([
  {
    path: "/auth",
    children: [
      { index: true, element: <Navigate to="/auth/login" replace /> },
      { path: "login", element: <LoginPage /> },
      { path: "register", element: <RegisterPage /> },
      { path: "forgot", element: <ForgotPasswordPage /> },
      { path: "reset/:token", element: <ResetPasswordPage /> },
      { path: "mfa", element: <MfaPage /> }
    ]
  },
  {
    element: <AuthGuard />,
    children: [
      {
        element: <AppLayout />,
        children: [
          { index: true, element: <Navigate to="/dashboard" replace /> },
          { path: "/dashboard", element: <DashboardEntryPage /> },
          { path: "/requests", element: <RequestsPage /> },
          { path: "/requests/:id", element: <RequestDetailPage /> },
          { path: "/objects", element: <ObjectsPage /> },
          { path: "/objects/:id", element: <ObjectDetailPage /> },
          {
            element: <RoleGuard roles={["CLIENT", "ADMIN"]} />,
            children: [
              { path: "/requests/new", element: <RequestFormPage /> }
            ]
          },
          {
            element: <RoleGuard roles={["ADMIN", "MANAGER", "DISPATCHER", "TECHNICIAN"]} />,
            children: [
              { path: "/requests/kanban", element: <RequestsKanbanPage /> },
              { path: "/requests/:id/edit", element: <RequestFormPage /> },
              { path: "/work-orders", element: <WorkOrdersPage /> },
              { path: "/work-orders/:id", element: <WorkOrderDetailPage /> },
              { path: "/sla/dashboard", element: <SlaDashboardPage /> },
              { path: "/sla/policies", element: <SlaPoliciesPage /> },
              { path: "/sla/breaches", element: <SlaDashboardPage /> },
              { path: "/sla/reports", element: <SimplePage title="SLA отчеты" /> },
              { path: "/escalations", element: <EscalationsPage /> },
              { path: "/escalations/rules", element: <EscalationRulesPage /> },
              { path: "/escalations/history", element: <SimplePage title="История эскалаций" /> },
              { path: "/analytics/overview", element: <AnalyticsPage /> },
              { path: "/analytics/requests", element: <AnalyticsPage /> },
              { path: "/analytics/staff", element: <AnalyticsPage /> },
              { path: "/analytics/objects", element: <AnalyticsPage /> },
              { path: "/analytics/reports", element: <AnalyticsPage /> },
              { path: "/objects/:id/history", element: <SimplePage title="История обслуживания объекта" /> }
            ]
          },
          {
            element: <RoleGuard roles={["ADMIN"]} />,
            children: [
              { path: "/objects/new", element: <ObjectCreatePage /> },
              { path: "/users", element: <UsersPage /> },
              { path: "/users/:id", element: <UsersPage /> },
              { path: "/roles", element: <RolesPage /> },
              { path: "/teams", element: <SimplePage title="Команды" /> }
            ]
          },
          { path: "/notifications", element: <NotificationsPage /> },
          { path: "/settings/profile", element: <SettingsPage section="profile" /> },
          { path: "/settings/security", element: <SettingsPage section="security" /> },
          { path: "/settings/notifications", element: <SettingsPage section="notifications" /> },
          { path: "/settings/appearance", element: <SettingsPage section="appearance" /> },
          { path: "/settings/system", element: <SettingsPage section="system" /> }
        ]
      }
    ]
  },
  { path: "*", element: <Navigate to="/dashboard" replace /> }
]);
