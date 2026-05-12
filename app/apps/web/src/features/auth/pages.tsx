import { Navigate } from "react-router-dom";
import type { ReactNode } from "react";
import { LoginForm } from "@/features/auth/LoginForm";

function AuthShell({ title, children }: { title: string; children: ReactNode }) {
  return (
    <main className="auth-page">
      <section className="auth-panel">
        <h1>{title}</h1>
        {children}
      </section>
    </main>
  );
}

export function LoginPage() {
  return (
    <AuthShell title="Вход в ServiceDesk">
      <LoginForm />
    </AuthShell>
  );
}

export function RegisterPage() {
  return <Navigate to="/auth/login" replace />;
}

export function ForgotPasswordPage() {
  return <Navigate to="/auth/login" replace />;
}

export function ResetPasswordPage() {
  return <Navigate to="/auth/login" replace />;
}

export function MfaPage() {
  return <Navigate to="/auth/login" replace />;
}
