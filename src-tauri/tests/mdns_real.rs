//! Descubrimiento mDNS con multicast real (T151, FR-010, `Historias.md` §7.1).
//!
//! Dos instancias en la misma máquina se anuncian y se buscan por la red de verdad. Va
//! `#[ignore]`: depende de que el equipo tenga una interfaz con multicast y de que el
//! firewall no lo bloquee (V-08), así que no debe hacer fallar una ejecución en una máquina
//! sin eso.
//!
//! ```text
//! cargo test --manifest-path src-tauri/Cargo.toml --test mdns_real -- --ignored --nocapture
//! ```

use networkbench_lib::discovery::{Descubrimiento, EquipoDescubierto};
use networkbench_lib::identity::InstanceIdentity;
use std::time::{Duration, Instant};
use uuid::Uuid;

fn esperar(limite: Duration, mut condicion: impl FnMut() -> bool) -> bool {
    let inicio = Instant::now();
    while inicio.elapsed() < limite {
        if condicion() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    condicion()
}

fn ve(d: &Descubrimiento, id: Uuid) -> Option<EquipoDescubierto> {
    d.equipos().into_iter().find(|e| e.instance_id == id)
}

#[test]
#[ignore = "usa multicast real; ejecutar con --ignored"]
fn dos_instancias_se_descubren_por_mdns_y_desaparecen_al_cerrarse() {
    let a = InstanceIdentity::generate("Equipo A".into()).unwrap();
    let b = InstanceIdentity::generate("Equipo B".into()).unwrap();

    let da = Descubrimiento::iniciar(&a, 7411, "0.1.0", 1).expect("publicar A");
    let db = Descubrimiento::iniciar(&b, 7412, "0.1.0", 1).expect("publicar B");

    assert!(
        esperar(Duration::from_secs(20), || ve(&da, b.instance_id).is_some()
            && ve(&db, a.instance_id).is_some()),
        "cada instancia debe ver a la otra; A ve {:?}, B ve {:?}",
        da.equipos(),
        db.equipos()
    );

    let visto_b = ve(&da, b.instance_id).unwrap();
    assert_eq!(visto_b.display_name, "Equipo B");
    assert_eq!(visto_b.fingerprint_declarada, b.fingerprint.to_lowercase());
    assert!(!visto_b.addresses.is_empty());
    assert!(
        visto_b.addresses.iter().all(|d| d.ends_with(":7412")),
        "el puerto anunciado es el de control de B: {:?}",
        visto_b.addresses
    );
    assert_eq!(visto_b.protocol_version, 1);

    // Nadie se ve a sí mismo como «otro equipo».
    assert!(ve(&da, a.instance_id).is_none());
    assert!(ve(&db, b.instance_id).is_none());

    // Un equipo que se cierra retira su anuncio: no sigue en la lista de los demás.
    drop(db);
    assert!(
        esperar(Duration::from_secs(20), || ve(&da, b.instance_id).is_none()),
        "B se cerró y debe desaparecer de la lista de A; A aún ve {:?}",
        da.equipos()
    );
}
