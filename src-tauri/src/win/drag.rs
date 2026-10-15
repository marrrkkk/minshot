//! Native file-drag source: a minimal Windows CF_HDROP `IDataObject`.
//!
//! Terminals (Windows Terminal, conhost, WezTerm, …) do not accept bitmap
//! drops — a file drop lands as text (the file path). OpenCode's TUI reads a
//! dropped PNG/JPEG/GIF/WebP *path* as an image attachment, so dragging the
//! temp PNG file is the correct integration. True pixel transfer into a
//! terminal is not possible; clipboard paste (Ctrl+V) is the pixel path.
//!
//! Threading: `DoDragDrop` runs a modal OLE loop and must live on the main
//! (UI) thread. This module hops there via `run_on_main_thread` and returns
//! immediately. A left-button guard avoids starting a drag after a fast click
//! already released the button (which would wedge the modal loop).

use std::path::PathBuf;

#[cfg(target_os = "windows")]
mod imp {
    use super::*;
    use std::sync::Mutex;
    use windows::{
        core::*,
        Win32::{
            Foundation::*,
            System::{
                Com::*,
                Memory::*,
                Ole::{
                    DoDragDrop, OleInitialize, CF_HDROP, DROPEFFECT, DROPEFFECT_COPY,
                    IDropSource, IDropSource_Impl,
                },
                SystemServices::{MK_LBUTTON, MODIFIERKEYS_FLAGS},
            },
            UI::{
                Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON},
                Shell::DROPFILES,
            },
        },
    };

    /// Pre-rendered DROPFILES payload: header + NUL-separated UTF-16 paths.
    #[implement(IDataObject)]
    pub(crate) struct FileDropData {
        pub(crate) payload: Vec<u8>,
    }

    impl FileDropData {
        pub(crate) fn new(files: &[PathBuf]) -> Self {
            use std::{iter::once, os::windows::ffi::OsStrExt};
            let mut wide: Vec<u16> = Vec::new();
            for f in files {
                wide.extend(f.as_os_str().encode_wide().chain(once(0)));
            }
            wide.push(0);
            let header_len = std::mem::size_of::<DROPFILES>();
            let mut payload = vec![0u8; header_len + wide.len() * 2];
            let header = DROPFILES {
                pFiles: header_len as u32,
                // pt, fNC zeroed by vec init
                fWide: TRUE,
                ..Default::default()
            };
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &header as *const DROPFILES as *const u8,
                    payload.as_mut_ptr(),
                    header_len,
                );
                std::ptr::copy_nonoverlapping(
                    wide.as_ptr() as *const u8,
                    payload.as_mut_ptr().add(header_len),
                    wide.len() * 2,
                );
            }
            Self { payload }
        }

        fn is_hdrop(pformatetc: *const FORMATETC) -> bool {
            unsafe {
                match pformatetc.as_ref() {
                    Some(f) => {
                        f.cfFormat == CF_HDROP.0
                            && f.dwAspect == DVASPECT_CONTENT.0
                            && (f.tymed & TYMED_HGLOBAL.0 as u32) != 0
                    }
                    None => false,
                }
            }
        }

        fn hglobal_for(&self) -> Result<HGLOBAL> {
            unsafe {
                let handle = GlobalAlloc(GMEM_MOVEABLE, self.payload.len())?;
                let ptr = GlobalLock(handle) as *mut u8;
                if ptr.is_null() {
                    let _ = GlobalFree(Some(handle));
                    return Err(Error::new(E_FAIL, "GlobalLock returned null"));
                }
                std::ptr::copy_nonoverlapping(
                    self.payload.as_ptr(),
                    ptr,
                    self.payload.len(),
                );
                let _ = GlobalUnlock(handle);
                Ok(handle)
            }
        }

        fn hdrop_format() -> FORMATETC {
            FORMATETC {
                cfFormat: CF_HDROP.0,
                ptd: std::ptr::null_mut(),
                dwAspect: DVASPECT_CONTENT.0,
                lindex: -1,
                tymed: TYMED_HGLOBAL.0 as u32,
            }
        }
    }

    #[allow(non_snake_case)]
    impl IDataObject_Impl for FileDropData_Impl {
        fn GetData(&self, pformatetc: *const FORMATETC) -> Result<STGMEDIUM> {
            if !FileDropData::is_hdrop(pformatetc) {
                return Err(Error::new(DV_E_FORMATETC, "unsupported format"));
            }
            Ok(STGMEDIUM {
                tymed: TYMED_HGLOBAL.0 as u32,
                u: STGMEDIUM_0 {
                    hGlobal: self.hglobal_for()?,
                },
                pUnkForRelease: std::mem::ManuallyDrop::new(None),
            })
        }

        fn GetDataHere(&self, _pformatetc: *const FORMATETC, _pmedium: *mut STGMEDIUM) -> Result<()> {
            Err(Error::new(E_NOTIMPL, "not implemented"))
        }

        fn QueryGetData(&self, pformatetc: *const FORMATETC) -> HRESULT {
            if FileDropData::is_hdrop(pformatetc) {
                S_OK
            } else {
                DV_E_FORMATETC
            }
        }

        fn GetCanonicalFormatEtc(
            &self,
            _pformatectin: *const FORMATETC,
            pformatetcout: *mut FORMATETC,
        ) -> HRESULT {
            unsafe {
                (*pformatetcout).ptd = std::ptr::null_mut();
            }
            E_NOTIMPL
        }

        fn SetData(
            &self,
            _pformatetc: *const FORMATETC,
            _pmedium: *const STGMEDIUM,
            _frelease: BOOL,
        ) -> Result<()> {
            Err(Error::new(E_NOTIMPL, "not implemented"))
        }

        fn EnumFormatEtc(&self, dwdirection: u32) -> Result<IEnumFORMATETC> {
            if dwdirection != DATADIR_GET.0 as u32 {
                return Err(Error::new(E_NOTIMPL, "not implemented"));
            }
            Ok(
                ComObject::new(FormatEtcEnumerator {
                    items: vec![FileDropData::hdrop_format()],
                    index: Mutex::new(0),
                })
                .to_interface(),
            )
        }

        fn DAdvise(
            &self,
            _pformatetc: *const FORMATETC,
            _advf: u32,
            _padvsink: Ref<'_, IAdviseSink>,
        ) -> Result<u32> {
            Err(Error::new(OLE_E_ADVISENOTSUPPORTED, "advise not supported"))
        }

        fn DUnadvise(&self, _dwconnection: u32) -> Result<()> {
            Err(Error::new(OLE_E_ADVISENOTSUPPORTED, "advise not supported"))
        }

        fn EnumDAdvise(&self) -> Result<IEnumSTATDATA> {
            Err(Error::new(OLE_E_ADVISENOTSUPPORTED, "advise not supported"))
        }
    }

    #[implement(IEnumFORMATETC)]
    struct FormatEtcEnumerator {
        items: Vec<FORMATETC>,
        index: Mutex<usize>,
    }

    #[allow(non_snake_case)]
    impl IEnumFORMATETC_Impl for FormatEtcEnumerator_Impl {
        fn Next(
            &self,
            celt: u32,
            rgelt: *mut FORMATETC,
            pceltfetched: *mut u32,
        ) -> HRESULT {
            let mut index = match self.index.lock() {
                Ok(g) => g,
                Err(_) => return E_FAIL,
            };
            let mut fetched = 0u32;
            while fetched < celt {
                if *index >= self.items.len() {
                    break;
                }
                unsafe {
                    *rgelt.add(fetched as usize) = self.items[*index];
                }
                *index += 1;
                fetched += 1;
            }
            if !pceltfetched.is_null() {
                unsafe {
                    *pceltfetched = fetched;
                }
            }
            if fetched == celt {
                S_OK
            } else {
                S_FALSE
            }
        }

        fn Skip(&self, celt: u32) -> Result<()> {
            let mut index = self.index.lock().map_err(|_| Error::new(E_FAIL, "lock failed"))?;
            *index = (*index + celt as usize).min(self.items.len());
            if *index < self.items.len() {
                Ok(())
            } else {
                // Past the end: S_FALSE, not a failure.
                Err(Error::new(S_FALSE, "end of enumeration"))
            }
        }

        fn Reset(&self) -> Result<()> {
            match self.index.lock() {
                Ok(mut g) => {
                    *g = 0;
                    Ok(())
                }
                Err(_) => Err(Error::new(E_FAIL, "lock failed")),
            }
        }

        fn Clone(&self) -> Result<IEnumFORMATETC> {
            let index = *self.index.lock().map_err(|_| Error::new(E_FAIL, "lock failed"))?;
            Ok(ComObject::new(FormatEtcEnumerator {
                items: self.items.clone(),
                index: Mutex::new(index),
            })
            .to_interface())
        }
    }

    #[implement(IDropSource)]
    struct DropSource;

    #[allow(non_snake_case)]
    impl IDropSource_Impl for DropSource_Impl {
        fn QueryContinueDrag(
            &self,
            fescapepressed: BOOL,
            grfkeystate: MODIFIERKEYS_FLAGS,
        ) -> HRESULT {
            if fescapepressed.as_bool() {
                DRAGDROP_S_CANCEL
            } else if (grfkeystate & MK_LBUTTON) == MODIFIERKEYS_FLAGS(0) {
                DRAGDROP_S_DROP
            } else {
                S_OK
            }
        }

        fn GiveFeedback(&self, _dweffect: DROPEFFECT) -> HRESULT {
            DRAGDROP_S_USEDEFAULTCURSORS
        }
    }

    fn button_down() -> bool {
        unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) < 0 }
    }

    pub fn start_on_main_thread(file: PathBuf) {
        if !button_down() {
            // Fast click released before the hop: no drag to perform.
            return;
        }
        unsafe {
            let _ = OleInitialize(Some(std::ptr::null_mut()));
        }
        let data: IDataObject =
            ComObject::new(FileDropData::new(std::slice::from_ref(&file))).to_interface();
        let source: IDropSource = ComObject::new(DropSource).to_interface();
        let mut effect = DROPEFFECT::default();
        unsafe {
            let _ = DoDragDrop(&data, &source, DROPEFFECT_COPY, &mut effect);
        }
    }
}

/// Kick off a native drag of `file` from the main window.
/// Hops to the main thread and returns immediately; the drag continues
/// modally there until the user drops or cancels.
pub fn start_file_drag_main_thread(app: tauri::AppHandle, file: PathBuf) -> Result<(), String> {
    if !file.is_file() {
        return Err(format!("nothing to drag (missing {})", file.display()));
    }
    #[cfg(target_os = "windows")]
    {
        app.run_on_main_thread(move || imp::start_on_main_thread(file))
            .map_err(|e| format!("failed to hop to main thread: {e}"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, file);
        Err("native drag is Windows-only".to_string())
    }
}

/// Render the HDROP payload for `paths` (test seam; production path goes
/// through [`start_file_drag_main_thread`]).
#[allow(dead_code)]
pub fn dropfiles_payload_for(paths: &[PathBuf]) -> Vec<u8> {
    #[cfg(target_os = "windows")]
    {
        imp::FileDropData::new(paths).payload
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = paths;
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The DROPFILES payload must be a valid HDROP: 20-byte header with
    /// pFiles=20, fWide=1, followed by NUL-terminated UTF-16 paths + final NUL.
    #[test]
    fn dropfiles_payload_layout() {
        let payload =
            dropfiles_payload_for(&[PathBuf::from(r"C:\Temp\minshot-a.png")]);
        assert!(payload.len() > 20);
        let p_files = u32::from_le_bytes(payload[0..4].try_into().unwrap());
        assert_eq!(p_files, 20);
        let f_wide = u32::from_le_bytes(payload[16..20].try_into().unwrap());
        assert_eq!(f_wide, 1);
        let wide_len = (payload.len() - 20) / 2;
        let mut units = Vec::with_capacity(wide_len);
        for i in 0..wide_len {
            units.push(u16::from_le_bytes(
                payload[20 + i * 2..22 + i * 2].try_into().unwrap(),
            ));
        }
        let text = String::from_utf16(&units).unwrap();
        let mut parts = text.split('\0');
        assert_eq!(parts.next(), Some(r"C:\Temp\minshot-a.png"));
        // Double-NUL terminator: the path plus two empty segments.
        assert_eq!(parts.next(), Some(""));
        assert_eq!(parts.next(), Some(""));
        assert_eq!(parts.next(), None);
    }
}
