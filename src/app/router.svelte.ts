export type AppRoute = "inicio" | "historial" | "ajustes" | "peers" | "session" | "results";

export class AppRouter {
  currentRoute = $state<AppRoute>("inicio");
  previousRoute = $state<AppRoute | null>(null);

  navigate(route: AppRoute): boolean {
    if (this.currentRoute === route) return true;
    this.previousRoute = this.currentRoute;
    this.currentRoute = route;
    return true;
  }

  canNavigate(target: AppRoute, isSessionActive: boolean): boolean {
    if (!isSessionActive) return true;
    // Si hay sesión activa, solo se permite volver a inicio con confirmación de cancelación
    return target === "inicio" || target === this.currentRoute;
  }
}

export const router = new AppRouter();
