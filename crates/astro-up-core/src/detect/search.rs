//! Windows Search index file lookup.
//!
//! Queries the Windows Search index to find files by name.
//! Returns instantly (< 100ms) when the index is available.
//! Falls back gracefully if the service is disabled.
//!
//! Uses ADODB COM objects to query the SystemIndex OLE DB provider,
//! which is the documented approach for querying Windows Search.

/// Find a file by filename using the Windows Search index.
///
/// Returns `Ok(Some(path))` if found, `Ok(None)` if not in the index,
/// or `Err` if the search service is unavailable.
#[cfg(windows)]
pub fn find_file(filename: &str) -> Result<Option<String>, String> {
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, IDispatch,
    };
    use windows::Win32::System::Variant::VARIANT;
    use windows::core::{BSTR, Interface};

    // COM must be initialized on this thread
    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };

    // Escape single quotes in filename
    let safe_name = filename.replace('\'', "''");

    let sql = format!(
        "SELECT TOP 1 System.ItemPathDisplay FROM SystemIndex \
         WHERE System.FileName = '{safe_name}' \
         AND scope='file:'"
    );

    tracing::debug!(filename, sql = sql.as_str(), "Windows Search query");

    // Create ADODB.Connection
    let conn: IDispatch = unsafe {
        CoCreateInstance(
            &windows::core::GUID::from_u128(0x00000514_0000_0010_8000_00AA006D2EA4), // ADODB.Connection
            None,
            CLSCTX_INPROC_SERVER,
        )
    }
    .map_err(|e| format!("cannot create ADODB.Connection: {e}"))?;

    // conn.Open("Provider=Search.CollatorDSO;Extended Properties='Application=Windows'")
    let conn_str =
        BSTR::from("Provider=Search.CollatorDSO;Extended Properties='Application=Windows'");
    let open_id = oleaut_id(&conn, "Open").map_err(|e| format!("cannot find Open method: {e}"))?;

    let args = [
        VARIANT::from(BSTR::new()), // Password
        VARIANT::from(BSTR::new()), // UserID
        VARIANT::from(0i32),        // Options
        VARIANT::from(conn_str),    // ConnectionString
    ];

    unsafe {
        invoke_method(&conn, open_id, &args).map_err(|e| format!("Connection.Open failed: {e}"))?;
    }

    // conn.Execute(sql)
    let exec_id =
        oleaut_id(&conn, "Execute").map_err(|e| format!("cannot find Execute method: {e}"))?;
    let sql_bstr = BSTR::from(sql.as_str());
    let exec_args = [
        VARIANT::from(0i32),     // Options
        VARIANT::default(),      // RecordsAffected (out, ignored)
        VARIANT::from(sql_bstr), // CommandText
    ];

    let rs_variant = unsafe {
        invoke_method(&conn, exec_id, &exec_args)
            .map_err(|e| format!("Connection.Execute failed: {e}"))?
    };

    // Get the recordset IDispatch
    let rs: IDispatch =
        unsafe { rs_variant.to_idispatch() }.map_err(|e| format!("cannot get recordset: {e}"))?;

    // Check EOF
    let eof_id = oleaut_id(&rs, "EOF").map_err(|e| format!("cannot find EOF property: {e}"))?;
    let eof_val =
        unsafe { get_property(&rs, eof_id) }.map_err(|e| format!("cannot get EOF: {e}"))?;

    let is_eof: bool = unsafe { eof_val.to_bool() }.map_err(|e| format!("EOF to bool: {e}"))?;

    if is_eof {
        // Close connection
        let close_id = oleaut_id(&conn, "Close").ok();
        if let Some(id) = close_id {
            unsafe { invoke_method(&conn, id, &[]).ok() };
        }
        return Ok(None);
    }

    // Get first field value: rs.Fields(0).Value
    let fields_id = oleaut_id(&rs, "Fields").map_err(|e| format!("cannot find Fields: {e}"))?;
    let fields_val =
        unsafe { get_property(&rs, fields_id) }.map_err(|e| format!("cannot get Fields: {e}"))?;
    let fields: IDispatch =
        unsafe { fields_val.to_idispatch() }.map_err(|e| format!("Fields to IDispatch: {e}"))?;

    let item_id = oleaut_id(&fields, "Item").map_err(|e| format!("cannot find Item: {e}"))?;
    let item_args = [VARIANT::from(0i32)];
    let field_val = unsafe { invoke_method(&fields, item_id, &item_args) }
        .map_err(|e| format!("Fields.Item(0) failed: {e}"))?;
    let field: IDispatch =
        unsafe { field_val.to_idispatch() }.map_err(|e| format!("Field to IDispatch: {e}"))?;

    let value_id =
        oleaut_id(&field, "Value").map_err(|e| format!("cannot find Value property: {e}"))?;
    let path_val =
        unsafe { get_property(&field, value_id) }.map_err(|e| format!("cannot get Value: {e}"))?;
    let path: BSTR = unsafe { path_val.to_bstr() }.map_err(|e| format!("Value to BSTR: {e}"))?;

    // Close
    let close_id = oleaut_id(&conn, "Close").ok();
    if let Some(id) = close_id {
        unsafe { invoke_method(&conn, id, &[]).ok() };
    }

    let result = path.to_string();
    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}

#[cfg(not(windows))]
#[allow(dead_code)] // called from #[cfg(windows)] block in pe.rs
pub fn find_file(_filename: &str) -> Result<Option<String>, String> {
    Err("Windows Search is only available on Windows".into())
}

// --- OLE Automation helpers ---

#[cfg(windows)]
use windows::Win32::System::Com::IDispatch;
#[cfg(windows)]
use windows::Win32::System::Variant::VARIANT;
#[cfg(windows)]
use windows::core::BSTR;

/// Get a DISPID for a method/property name.
#[cfg(windows)]
fn oleaut_id(obj: &IDispatch, name: &str) -> Result<i32, String> {
    use windows::core::PCWSTR;

    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let names = [PCWSTR(wide.as_ptr())];
    let mut ids = [0i32];

    unsafe {
        obj.GetIDsOfNames(
            &windows::core::GUID::zeroed(),
            names.as_ptr(),
            1,
            0x0400, // LOCALE_SYSTEM_DEFAULT
            ids.as_mut_ptr(),
        )
    }
    .map_err(|e| format!("GetIDsOfNames({name}): {e}"))?;

    Ok(ids[0])
}

/// Invoke a method with the given arguments (args in REVERSE order for IDispatch).
#[cfg(windows)]
unsafe fn invoke_method(obj: &IDispatch, dispid: i32, args: &[VARIANT]) -> Result<VARIANT, String> {
    use windows::Win32::System::Com::{DISPATCH_METHOD, DISPPARAMS};

    // IDispatch expects args in reverse order
    let mut reversed: Vec<VARIANT> = args.iter().rev().cloned().collect();

    let mut params = DISPPARAMS {
        rgvarg: if reversed.is_empty() {
            std::ptr::null_mut()
        } else {
            reversed.as_mut_ptr()
        },
        rgdispidNamedArgs: std::ptr::null_mut(),
        cArgs: reversed.len() as u32,
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
}

/// Get a property value.
#[cfg(windows)]
unsafe fn get_property(obj: &IDispatch, dispid: i32) -> Result<VARIANT, String> {
    use windows::Win32::System::Com::{DISPATCH_PROPERTYGET, DISPPARAMS};

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
    .map_err(|e| format!("Invoke({dispid}): {e}"))?;

    Ok(result)
}
