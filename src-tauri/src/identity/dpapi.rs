use std::io::{Error, Result};

#[cfg(windows)]
pub fn protect_bytes(data: &[u8]) -> Result<Vec<u8>> {
    use windows::Win32::Foundation::HLOCAL;
    use windows::Win32::Foundation::LocalFree;
    use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptProtectData};

    let in_blob = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    let mut out_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    unsafe {
        CryptProtectData(&in_blob, None, None, None, None, 0, &mut out_blob)
            .map_err(|e| Error::other(format!("CryptProtectData falló: {}", e)))?;

        let slice = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize);
        let protected = slice.to_vec();
        let _ = LocalFree(Some(HLOCAL(out_blob.pbData as *mut _)));
        Ok(protected)
    }
}

#[cfg(windows)]
pub fn unprotect_bytes(protected_data: &[u8]) -> Result<Vec<u8>> {
    use windows::Win32::Foundation::HLOCAL;
    use windows::Win32::Foundation::LocalFree;
    use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptUnprotectData};

    let in_blob = CRYPT_INTEGER_BLOB {
        cbData: protected_data.len() as u32,
        pbData: protected_data.as_ptr() as *mut u8,
    };
    let mut out_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };

    unsafe {
        CryptUnprotectData(&in_blob, None, None, None, None, 0, &mut out_blob)
            .map_err(|e| Error::other(format!("CryptUnprotectData falló: {}", e)))?;

        let slice = std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize);
        let unprotected = slice.to_vec();
        let _ = LocalFree(Some(HLOCAL(out_blob.pbData as *mut _)));
        Ok(unprotected)
    }
}

#[cfg(not(windows))]
pub fn protect_bytes(data: &[u8]) -> Result<Vec<u8>> {
    Ok(data.to_vec())
}

#[cfg(not(windows))]
pub fn unprotect_bytes(protected_data: &[u8]) -> Result<Vec<u8>> {
    Ok(protected_data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpapi_roundtrip() {
        let secret = b"my_super_secret_ed25519_key_bytes";
        let protected = protect_bytes(secret).expect("Cifrado DPAPI");
        assert_ne!(&protected[..], &secret[..]);

        let decrypted = unprotect_bytes(&protected).expect("Descifrado DPAPI");
        assert_eq!(&decrypted[..], &secret[..]);
    }
}
