use crate::firewall::{
    FirewallHelperClient, FirewallHelperRequest, FirewallInspection, FirewallInspector,
};
use crate::ipc::response::IpcResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectRuleRequest {
    pub rule_name: String,
    pub expected_port_range: String,
    pub expected_proto: String,
}

#[tauri::command]
pub async fn firewall_inspect(request: InspectRuleRequest) -> IpcResult<FirewallInspection> {
    let result = FirewallInspector::inspect_rule(
        &request.rule_name,
        &request.expected_port_range,
        &request.expected_proto,
    );
    IpcResult::ok(result)
}

#[tauri::command]
pub async fn firewall_apply(rules: Vec<FirewallHelperRequest>) -> IpcResult<()> {
    match FirewallHelperClient::apply_rules(&rules) {
        Ok(()) => IpcResult::ok(()),
        Err(e) => IpcResult::err(e),
    }
}

// ---------------------------------------------------------------------------------------
// Reglas de NetworkBench en el Firewall de Windows (Historias.md §14.1 y §14.6)
//
// Ningún comando recibe reglas del frontend: el conjunto lo calcula el backend a partir de
// los ajustes y de dónde está el ejecutable, y solo eso se le entrega al helper elevado.
// ---------------------------------------------------------------------------------------

use crate::app::AppState;
use crate::errors::{AppError, ErrorCode};
use crate::firewall::RuleStatus;
use crate::firewall::estado::{self, EstadoRegla};
use crate::firewall::reglas::{self, Entorno};
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InformeReglas {
    pub reglas: Vec<EstadoRegla>,
    /// Puerto de control con el que se han calculado (el personalizado o el de fábrica).
    pub puerto_control: u16,
    /// Existe el helper elevado: sin él no se pueden crear ni eliminar reglas desde la app.
    pub ayudante_disponible: bool,
}

fn error_interno(detalle: String) -> AppError {
    AppError::from_code(ErrorCode::InternalError).with_diagnostic_id(detalle)
}

fn entorno(state: &AppState) -> Result<Entorno, AppError> {
    let ajustes = state.settings.get();
    let puerto = ajustes
        .custom_control_port
        .unwrap_or(crate::discovery::CONTROL_PORT_DEFAULT);
    Entorno::actual(puerto, ajustes.mdns_enabled)
        .ok_or_else(|| error_interno("No se pudo localizar el ejecutable".into()))
}

/// Lee el sistema (PowerShell, hasta unos segundos): siempre fuera del hilo asíncrono.
async fn informe(entorno: Entorno) -> Result<InformeReglas, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let esperadas = reglas::esperadas(&entorno);
        let reglas = estado::consultar(&esperadas).map_err(error_interno)?;
        Ok(InformeReglas {
            reglas,
            puerto_control: entorno.puerto_control,
            ayudante_disponible: FirewallHelperClient::get_helper_path().is_some(),
        })
    })
    .await
    .map_err(|e| error_interno(e.to_string()))?
}

#[tauri::command]
pub async fn firewall_rules_status(
    state: State<'_, AppState>,
) -> Result<IpcResult<InformeReglas>, String> {
    let resultado = match entorno(&state) {
        Ok(e) => informe(e).await,
        Err(e) => Err(e),
    };
    Ok(match resultado {
        Ok(i) => IpcResult::ok(i),
        Err(e) => IpcResult::err(e),
    })
}

/// «Crear las que faltan»: añade (o recrea, si el helper las encuentra distintas) las reglas
/// que no están presentes y cuyo programa existe. Pide UAC; la aplicación no se eleva.
#[tauri::command]
pub async fn firewall_rules_create(
    state: State<'_, AppState>,
) -> Result<IpcResult<InformeReglas>, String> {
    let entorno = match entorno(&state) {
        Ok(e) => e,
        Err(e) => return Ok(IpcResult::err(e)),
    };

    let entorno_para_helper = entorno.clone();
    let aplicado = tauri::async_runtime::spawn_blocking(move || -> Result<(), AppError> {
        let esperadas = reglas::esperadas(&entorno_para_helper);
        let estados = estado::consultar(&esperadas).map_err(error_interno)?;
        let peticiones: Vec<_> = estados
            .iter()
            .filter(|s| s.estado != RuleStatus::Present && s.programa_existe)
            .map(|s| reglas::a_peticion(&s.regla, "add"))
            .collect();
        if peticiones.is_empty() {
            return Ok(());
        }
        FirewallHelperClient::apply_rules(&peticiones)
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Err(e) = aplicado {
        return Ok(IpcResult::err(e));
    }
    Ok(match informe(entorno).await {
        Ok(i) => IpcResult::ok(i),
        Err(e) => IpcResult::err(e),
    })
}

/// «Eliminar todas»: quita las reglas de la aplicación que existan. El helper falla si se
/// le pide borrar una que no existe, así que solo se piden las que están.
#[tauri::command]
pub async fn firewall_rules_remove(
    state: State<'_, AppState>,
) -> Result<IpcResult<InformeReglas>, String> {
    let entorno = match entorno(&state) {
        Ok(e) => e,
        Err(e) => return Ok(IpcResult::err(e)),
    };

    let aplicado = tauri::async_runtime::spawn_blocking(move || -> Result<(), AppError> {
        let presentes = estado::existentes().map_err(error_interno)?;
        let peticiones: Vec<_> = reglas::NOMBRES_CONOCIDOS
            .iter()
            .filter(|n| presentes.iter().any(|p| p == *n))
            .map(|n| FirewallHelperRequest {
                operation: "remove".into(),
                rule_name: (*n).to_string(),
                protocol: String::new(),
                port_range: String::new(),
                program: String::new(),
                profiles: vec![],
            })
            .collect();
        if peticiones.is_empty() {
            return Ok(());
        }
        FirewallHelperClient::apply_rules(&peticiones)
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Err(e) = aplicado {
        return Ok(IpcResult::err(e));
    }
    Ok(match informe(entorno).await {
        Ok(i) => IpcResult::ok(i),
        Err(e) => IpcResult::err(e),
    })
}
