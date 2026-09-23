use std::io::{Error, Result};

#[cfg(windows)]
pub struct JobObject {
    handle: windows::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl JobObject {
    pub fn create_kill_on_close() -> Result<Self> {
        use windows::Win32::System::JobObjects::{
            CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject,
        };

        unsafe {
            let handle = CreateJobObjectW(None, None)
                .map_err(|e| Error::other(format!("CreateJobObjectW falló: {}", e)))?;

            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
            .map_err(|e| {
                let _ = windows::Win32::Foundation::CloseHandle(handle);
                Error::other(format!("SetInformationJobObject falló: {}", e))
            })?;

            Ok(Self { handle })
        }
    }

    pub fn assign_process(&self, process_handle: windows::Win32::Foundation::HANDLE) -> Result<()> {
        use windows::Win32::System::JobObjects::AssignProcessToJobObject;

        unsafe {
            AssignProcessToJobObject(self.handle, process_handle)
                .map_err(|e| Error::other(format!("AssignProcessToJobObject falló: {}", e)))?;
            Ok(())
        }
    }
}

#[cfg(windows)]
impl Drop for JobObject {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::Foundation::CloseHandle(self.handle);
        }
    }
}

#[cfg(not(windows))]
pub struct JobObject;

#[cfg(not(windows))]
impl JobObject {
    pub fn create_kill_on_close() -> Result<Self> {
        Ok(Self)
    }
    pub fn assign_process(&self, _handle: usize) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_job_object() {
        let job = JobObject::create_kill_on_close();
        assert!(
            job.is_ok(),
            "Debe crear el Job Object con KILL_ON_JOB_CLOSE"
        );
    }
}
