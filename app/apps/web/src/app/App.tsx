import { RouterProvider } from "react-router-dom";
import { AuthProvider } from "@/app/providers/AuthProvider";
import { ThemeProvider } from "@/app/providers/ThemeProvider";
import { WebSocketProvider } from "@/app/providers/WebSocketProvider";
import { router } from "@/app/router";

export function App() {
  return (
    <ThemeProvider>
      <AuthProvider>
        <WebSocketProvider>
          <RouterProvider router={router} />
        </WebSocketProvider>
      </AuthProvider>
    </ThemeProvider>
  );
}
