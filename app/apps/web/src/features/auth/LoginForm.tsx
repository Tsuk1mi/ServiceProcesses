import { FormEvent, useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { useAuth } from "@/app/providers/AuthProvider";

export function LoginForm() {
  const [username, setUsername] = useState("admin");
  const [password, setPassword] = useState("admin");
  const [error, setError] = useState("");
  const { login } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    setError("");
    if (!username || !password) {
      setError("Введите логин и пароль.");
      return;
    }
    try {
      await login(username, password);
      navigate((location.state as { from?: { pathname?: string } } | null)?.from?.pathname ?? "/dashboard", { replace: true });
    } catch {
      setError("Не удалось выполнить вход. Проверьте логин и пароль.");
    }
  }

  return (
    <form className="auth-form" onSubmit={handleSubmit}>
      <label>
        Логин
        <input value={username} type="text" onChange={(event) => setUsername(event.target.value)} />
      </label>
      <label>
        Пароль
        <input value={password} type="password" onChange={(event) => setPassword(event.target.value)} />
      </label>
      <p className="form-hint">Доступна одна учетная запись: `admin` / `admin`.</p>
      {error ? <p className="form-error">{error}</p> : null}
      <button className="primary-button" type="submit">Войти</button>
    </form>
  );
}
