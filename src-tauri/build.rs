use std::process::Command;

/// Hash corto y fecha del commit con el que se compiló, para distinguir en Ajustes → Acerca
/// de si dos ordenadores están corriendo el mismo build (hallazgo real del propietario: la
/// versión de `Cargo.toml` no cambia entre sesiones de desarrollo, así que no sirve para
/// eso). `env!("CARGO_PKG_VERSION")` sigue siendo la versión de producto; esto es aparte.
///
/// Sin repositorio git (un tarball de código fuente sin `.git`, por ejemplo) o sin `git` en
/// el PATH, cae a "desconocido" en vez de romper la compilación: esto es diagnóstico, no
/// algo de lo que dependa el arranque.
fn git_output(args: &[&str]) -> Option<String> {
    let salida = Command::new("git").args(args).output().ok()?;
    if !salida.status.success() {
        return None;
    }
    let texto = String::from_utf8(salida.stdout).ok()?;
    let texto = texto.trim();
    if texto.is_empty() {
        None
    } else {
        Some(texto.to_string())
    }
}

fn main() {
    let hash =
        git_output(&["rev-parse", "--short=10", "HEAD"]).unwrap_or_else(|| "desconocido".into());
    let fecha =
        git_output(&["log", "-1", "--format=%cI"]).unwrap_or_else(|| "fecha desconocida".into());
    // "--porcelain" con salida vacía = árbol de trabajo limpio en el momento de compilar.
    let sucio = match git_output(&["status", "--porcelain"]) {
        Some(s) => !s.is_empty(),
        None => false,
    };

    println!("cargo:rustc-env=NB_GIT_HASH={hash}");
    println!("cargo:rustc-env=NB_GIT_DATE={fecha}");
    println!("cargo:rustc-env=NB_GIT_DIRTY={sucio}");
    // Sin esto, cargo no vuelve a ejecutar este script (y por tanto no refresca el hash) a
    // menos que cambie algún fichero rastreado por Cargo; un commit nuevo sin tocar código
    // no dispara una recompilación y el hash quedaría congelado en el de la última vez que
    // sí cambió algo.
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/index");

    tauri_build::build()
}
