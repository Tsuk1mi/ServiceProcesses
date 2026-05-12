import { FormEvent, useState } from "react";
import { useNavigate } from "react-router-dom";

export function MfaForm() {
  const [code, setCode] = useState("");
  const navigate = useNavigate();

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    if (code.length >= 6) {
      navigate("/dashboard");
    }
  }

  return (
    <form className="auth-form" onSubmit={handleSubmit}>
      <label>
        Код TOTP
        <input value={code} inputMode="numeric" maxLength={6} onChange={(event) => setCode(event.target.value)} />
      </label>
      <button className="primary-button" type="submit">Подтвердить</button>
    </form>
  );
}
