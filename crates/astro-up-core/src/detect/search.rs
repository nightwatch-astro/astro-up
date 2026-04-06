//! Windows Search index file lookup.
//!
//! Queries the Windows Search index to find files by name.
//! Uses ADODB COM objects via `IDispatch` to query the SystemIndex
//! OLE DB provider. Returns instantly (< 100ms) when the index is
//! available. Falls back gracefully if the service is disabled.

/// Find a file by filename using the Windows Search index.
///
/// Returns `Ok(Some(path))` if found, `Ok(None)` if not in the index,
/// or `Err` if the search service is unavailable.
#[cfg(windows)]
pub fn find_file(filename: &str) -> Result<Option<String>, String> {
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
        DISPATCH_METHOD, DISPATCH_PROPERTYGET, DISPPARAMS, IDispatch,
    };
    use windows::Win32::System::Variant::VARIANT;
    use windows::core::{BSTR, Interface, PCWSTR};

    // COM must be initialized on this thread
    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };

    let safe_name = filename.replace('\'', "''");
    let sql = format!(
        "SELECT TOP 1 System.ItemPathDisplay FROM SystemIndex \
         WHERE System.FileName = '{safe_name}' AND scope='file:'"
    );

    tracing::debug!(filename, sql = sql.as_str(), "Windows Search query");

    // Helper: get DISPID for a name
    let get_id = |obj: &IDispatch, name: &str| -> Result<i32, String> {
        let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let names = [PCWSTR(wide.as_ptr())];
        let mut ids = [0i32];
        unsafe {
            obj.GetIDsOfNames(
                &windows::core::GUID::zeroed(),
                names.as_ptr(),
                1,
                0x0400,
                ids.as_mut_ptr(),
            )
        }
        .map_err(|e| format!("GetIDsOfNames({name}): {e}"))?;
        Ok(ids[0])
    };

    // Helper: invoke method
    let invoke = |obj: &IDispatch, dispid: i32, args: &mut [VARIANT]| -> Result<VARIANT, String> {
        let mut params = DISPPARAMS {
            rgvarg: if args.is_empty() {
                std::ptr::null_mut()
            } else {
                args.as_mut_ptr()
            },
            rgdispidNamedArgs: std::ptr::null_mut(),
            cArgs: args.len() as u32,
            cNamedArgs: 0,
        };
        let mut result = VARIANT::default();
        unsafe {
            obj.Invoke(
                dispid,
                &windows::core::GUID::zeroed(),
                0x0400,
                DISPATCH_METHOD,
                &mut params,
                Some(&mut result),
                None,
                None,
            )
        }
        .map_err(|e| format!("Invoke({dispid}): {e}"))?;
        Ok(result)
    };

    // Helper: get property
    let get_prop = |obj: &IDispatch, dispid: i32| -> Result<VARIANT, String> {
        let mut params = DISPPARAMS::default();
        let mut result = VARIANT::default();
        unsafe {
            obj.Invoke(
                dispid,
                &windows::core::GUID::zeroed(),
                0x0400,
                DISPATCH_PROPERTYGET,
                &mut params,
                Some(&mut result),
                None,
                None,
            )
        }
        .map_err(|e| format!("GetProp({dispid}): {e}"))?;
        Ok(result)
    };

    // ADODB.Connection CLSID
    let adodb_clsid = windows::core::GUID::from_u128(0x00000514_0000_0010_8000_00AA006D2EA4);

    let conn: IDispatch = unsafe { CoCreateInstance(&adodb_clsid, None, CLSCTX_INPROC_SERVER) }
        .map_err(|e| format!("cannot create ADODB.Connection: {e}"))?;

    // conn.Open(connStr, "", "", 0) — args in reverse order for IDispatch
    let open_id = get_id(&conn, "Open")?;
    let conn_str =
        BSTR::from("Provider=Search.CollatorDSO;Extended Properties='Application=Windows'");
    let mut open_args = [
        VARIANT::from(0i32),        // Options
        VARIANT::from(BSTR::new()), // Password
        VARIANT::from(BSTR::new()), // UserID
        VARIANT::from(conn_str),    // ConnectionString
    ];
    invoke(&conn, open_id, &mut open_args).map_err(|e| format!("Connection.Open failed: {e}"))?;

    // conn.Execute(sql)
    let exec_id = get_id(&conn, "Execute")?;
    let mut exec_args = [
        VARIANT::from(0i32),                     // Options
        VARIANT::default(),                      // RecordsAffected
        VARIANT::from(BSTR::from(sql.as_str())), // CommandText
    ];
    let rs_variant = invoke(&conn, exec_id, &mut exec_args)
        .map_err(|e| format!("Connection.Execute failed: {e}"))?;

    // Extract recordset IDispatch
    let rs = IDispatch::try_from(&rs_variant).map_err(|e| format!("cannot get recordset: {e}"))?;

    // Check EOF
    let eof_id = get_id(&rs, "EOF")?;
    let eof_val = get_prop(&rs, eof_id)?;
    let is_eof = bool::try_from(&eof_val).unwrap_or(true);

    if is_eof {
        let _ = get_id(&conn, "Close")
            .ok()
            .map(|id| invoke(&conn, id, &mut []));
        return Ok(None);
    }

    // rs.Fields(0).Value
    let fields_id = get_id(&rs, "Fields")?;
    let fields_val = get_prop(&rs, fields_id)?;
    let fields = IDispatch::try_from(&fields_val).map_err(|e| format!("Fields: {e}"))?;

    let item_id = get_id(&fields, "Item")?;
    let mut item_args = [VARIANT::from(0i32)];
    let field_val =
        invoke(&fields, item_id, &mut item_args).map_err(|e| format!("Fields.Item(0): {e}"))?;
    let field = IDispatch::try_from(&field_val).map_err(|e| format!("Field: {e}"))?;

    let value_id = get_id(&field, "Value")?;
    let path_val = get_prop(&field, value_id)?;
    let path = BSTR::try_from(&path_val).map_err(|e| format!("Value to BSTR: {e}"))?;

    let _ = get_id(&conn, "Close")
        .ok()
        .map(|id| invoke(&conn, id, &mut []));

    let result = path.to_string();
    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}

#[cfg(not(windows))]
#[allow(dead_code)]
pub fn find_file(_filename: &str) -> Result<Option<String>, String> {
    Err("Windows Search is only available on Windows".into())
}
