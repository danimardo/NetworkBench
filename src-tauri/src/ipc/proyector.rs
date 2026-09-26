//! Mantiene al día el snapshot de la aplicación (T150, `contracts/ipc.md`).
//!
//! Hasta esta tarea el snapshot se construía una vez al arrancar y nadie lo actualizaba:
//! `isSessionActive`, `activeSessionId`, `peersCount`, el idioma y el tema quedaban
//! congelados, y la interfaz, que decide con ellos, nunca veía una sesión en marcha.
//!
//! El proyector **recalcula desde las fuentes** (ajustes, máquina de estados, base de
//! datos) en vez de aplicar parches por evento: así un aviso perdido o repetido no puede
//! dejar el snapshot en un estado que ninguna fuente respalda. `Notify` funde avisos
//! seguidos en uno y no pierde ninguno.

use super::events::EmisorDeEventos;
use super::snapshot::{AppSnapshot, SnapshotManager};
use crate::control::service::SessionService;
use crate::history::database::Database;
use crate::history::peers::contar_peers;
use crate::settings::{SettingsStore, ThemeMode};
use std::sync::Arc;
use tokio::sync::Notify;

pub struct ProyectorDeSnapshot {
    pub snapshot: Arc<SnapshotManager>,
    pub settings: Arc<SettingsStore>,
    pub session_service: Arc<SessionService>,
    pub database: Arc<Database>,
    pub aviso: Arc<Notify>,
}

impl ProyectorDeSnapshot {
    async fn proyectar(&self) -> AppSnapshot {
        let actual = self.snapshot.get_snapshot();
        let preferencias = self.settings.get();
        let estado = self.session_service.current_state().await;
        let activa = estado.is_active();
        let id_activa = if activa {
            self.session_service.active_session_id().await
        } else {
            None
        };
        // Si la base de datos no responde se conserva la cifra anterior: mejor un dato
        // ligeramente viejo que inventar un cero.
        let equipos = self
            .database
            .connection()
            .lock()
            .ok()
            .and_then(|db| contar_peers(&db).ok())
            .unwrap_or(actual.peers_count);

        AppSnapshot {
            locale: preferencias.locale,
            theme: match preferencias.theme {
                ThemeMode::System => "system",
                ThemeMode::Light => "light",
                ThemeMode::Dark => "dark",
            }
            .to_string(),
            is_session_active: activa,
            active_session_id: id_activa.map(|id| id.to_string()),
            peers_count: equipos,
            ..actual
        }
    }

    /// Recalcula y, si algo cambió, publica el nuevo snapshot con la revisión siguiente.
    pub async fn refrescar(&self) -> Option<AppSnapshot> {
        self.snapshot.sincronizar(self.proyectar().await)
    }

    /// Espera avisos y emite el snapshot cada vez que cambia. No termina nunca: vive lo
    /// que vive la aplicación.
    pub async fn vigilar(self: Arc<Self>, emisor: Arc<dyn EmisorDeEventos>) {
        loop {
            self.aviso.notified().await;
            if let Some(nuevo) = self.refrescar().await
                && let Err(e) = emisor.emitir_snapshot(&nuevo)
            {
                tracing::warn!("{e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::domain::SessionState;
    use crate::model::peer::Peer;
    use crate::model::plan::BenchmarkPlan;
    use crate::sampling::aggregate::SampleBatch;
    use std::sync::Mutex;
    use std::time::Duration;
    use uuid::Uuid;

    #[derive(Default)]
    struct Registro(Mutex<Vec<AppSnapshot>>);

    impl EmisorDeEventos for Registro {
        fn emitir_muestras(&self, _batch: &SampleBatch) -> Result<(), String> {
            Ok(())
        }
        fn emitir_snapshot(&self, snapshot: &AppSnapshot) -> Result<(), String> {
            self.0.lock().unwrap().push(snapshot.clone());
            Ok(())
        }
    }

    fn proyector() -> (Arc<ProyectorDeSnapshot>, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("nb-proyector-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let aviso = Arc::new(Notify::new());
        let p = Arc::new(ProyectorDeSnapshot {
            snapshot: Arc::new(SnapshotManager::default()),
            settings: Arc::new(SettingsStore::new(dir.join("settings.json"))),
            session_service: Arc::new(SessionService::con_aviso(Arc::clone(&aviso))),
            database: Arc::new(Database::open(dir.join("history.db")).unwrap()),
            aviso,
        });
        (p, dir)
    }

    fn equipo(n: u128) -> Peer {
        Peer::new(
            Uuid::new_v4(),
            format!("Equipo {n}"),
            format!("{n:064x}"),
            vec!["192.168.1.9:7411".into()],
        )
        .unwrap()
    }

    async fn esperar_emisiones(registro: &Registro, n: usize) {
        for _ in 0..100 {
            if registro.0.lock().unwrap().len() >= n {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!(
            "se esperaban {n} emisiones, hubo {}",
            registro.0.lock().unwrap().len()
        );
    }

    #[tokio::test]
    async fn una_sesion_que_empieza_y_termina_se_refleja_en_el_snapshot() {
        let (p, dir) = proyector();
        let registro = Arc::new(Registro::default());
        tokio::spawn(Arc::clone(&p).vigilar(registro.clone()));

        assert!(!p.snapshot.get_snapshot().is_session_active);

        let id = p
            .session_service
            .start_session(equipo(7), BenchmarkPlan::new_standard_tcp(5001))
            .await
            .unwrap();

        esperar_emisiones(&registro, 1).await;
        let al_empezar = registro.0.lock().unwrap()[0].clone();
        assert!(al_empezar.is_session_active);
        assert_eq!(al_empezar.active_session_id, Some(id.to_string()));
        assert_eq!(al_empezar.revision, 2);

        p.session_service
            .transition_to(SessionState::Failed, id)
            .await
            .unwrap();
        esperar_emisiones(&registro, 2).await;
        let al_terminar = registro.0.lock().unwrap()[1].clone();
        assert!(!al_terminar.is_session_active);
        assert_eq!(al_terminar.active_session_id, None);
        assert!(al_terminar.revision > al_empezar.revision);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn un_aviso_sin_cambios_no_emite_nada() {
        let (p, dir) = proyector();
        let registro = Arc::new(Registro::default());
        tokio::spawn(Arc::clone(&p).vigilar(registro.clone()));

        p.aviso.notify_one();
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(registro.0.lock().unwrap().is_empty());
        assert_eq!(p.snapshot.current_revision(), 1);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn un_equipo_nuevo_sube_el_contador() {
        let (p, dir) = proyector();
        assert_eq!(p.snapshot.get_snapshot().peers_count, 0);

        crate::history::upsert_peer(&p.database.connection().lock().unwrap(), &equipo(9)).unwrap();

        let nuevo = p.refrescar().await.expect("el contador cambió");
        assert_eq!(nuevo.peers_count, 1);
        assert!(p.refrescar().await.is_none());

        let _ = std::fs::remove_dir_all(dir);
    }
}
