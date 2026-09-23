use std::env;
use std::fs;
use std::process::Command;
use networkbench_lib::control::cleanup::CleanupCoordinator;
use networkbench_lib::engine::ntttcp::job_object::JobObject;

#[test]
fn test_cleanup_terminates_registered_child_process() {
    let coordinator = CleanupCoordinator::new();

    // Lanzar un proceso que duerma 30 segundos
    #[cfg(target_os = "windows")]
    let mut child = Command::new("powershell")
        .args(["-NoProfile", "-Command", "Start-Sleep -Seconds 30"])
        .spawn()
        .expect("Debe lanzar proceso de prueba");

    #[cfg(not(target_os = "windows"))]
    let mut child = Command::new("sleep")
        .arg("30")
        .spawn()
        .expect("Debe lanzar proceso de prueba");

    let pid = child.id();
    coordinator.register_process(pid);

    // Ejecutar cleanup
    coordinator.cleanup_all();

    // Comprobar que el proceso terminó
    let status = child.wait().expect("El proceso debe haber terminado");
    assert!(!status.success());
}

#[test]
fn test_cleanup_removes_temporary_files() {
    let coordinator = CleanupCoordinator::new();
    let temp_file1 = env::temp_dir().join(format!("test_cleanup_f1_{}.tmp", std::process::id()));
    let temp_file2 = env::temp_dir().join(format!("test_cleanup_f2_{}.tmp", std::process::id()));

    fs::write(&temp_file1, b"temp 1").unwrap();
    fs::write(&temp_file2, b"temp 2").unwrap();

    coordinator.register_temp_file(temp_file1.clone());
    coordinator.register_temp_file(temp_file2.clone());

    assert!(temp_file1.exists());
    assert!(temp_file2.exists());

    coordinator.cleanup_all();

    assert!(!temp_file1.exists());
    assert!(!temp_file2.exists());
}

#[test]
fn test_job_object_creation_and_registration() {
    let job = JobObject::create_kill_on_close().expect("Debe crear Job Object en Windows");

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::io::AsRawHandle;
        let mut child = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Start-Sleep -Seconds 10"])
            .spawn()
            .expect("Debe lanzar proceso hijo");

        let handle = windows::Win32::Foundation::HANDLE(child.as_raw_handle());
        let res = job.assign_process(handle);
        assert!(res.is_ok(), "Debe asignar proceso al Job Object");

        let _ = child.kill();
    }
}
