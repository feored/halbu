use log::error;

pub(crate) fn u32_from(slice: &[u8], name: &'static str) -> u32 {
    u32::from_le_bytes(match slice.try_into() {
        Ok(res) => res,
        Err(e) => {
            error!(
                "Reference: {0}:{1} (Failed to coerce [u8;4] from bytes: {2:?})",
                name,
                e.to_string(),
                slice
            );
            [0; 4]
        }
    })
}

pub(crate) fn u16_from(slice: &[u8], name: &'static str) -> u16 {
    u16::from_le_bytes(match slice.try_into() {
        Ok(res) => res,
        Err(e) => {
            error!(
                "Reference: {0}: {1} (Failed to coerce [u8;2] from bytes: {2:?})",
                name,
                e.to_string(),
                slice
            );
            [0; 2]
        }
    })
}
