use crate::error::{LighterError, Result};
use crate::models::common::OrderType;
use crate::models::order::TimeInForce;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use serde_json;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_longlong};
use std::path::PathBuf;
use std::sync::Arc;

/// Result type from the Go binary - matches Python's StrOrErr
#[repr(C)]
pub struct StrOrErr {
    pub str: *mut c_char, // Note: field name is 'str' not 'value'
    pub err: *mut c_char, // Note: field name is 'err' not 'error'
}

/// Transaction structure returned by the signer
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SignedTransaction {
    #[serde(rename = "ID")]
    pub id: String,
    pub sequence: u64,
    pub message_to_sign: String,
    pub signature: Option<String>,
    pub transaction: serde_json::Value,
}

/// FFI Signer that uses the native Go binaries
pub struct FFISigner {
    library: Arc<Library>,
    url: String,
    private_key: String,
    chain_id: c_int,
    api_key_index: c_int,
    account_index: c_int,
}

impl FFISigner {
    /// Create a new FFI signer
    pub fn new(
        url: &str,
        private_key: &str,
        api_key_index: i32,
        account_index: i32,
    ) -> Result<Self> {
        // Determine chain_id from URL
        let chain_id = if url.contains("mainnet") { 304 } else { 300 };

        // Determine the library path based on platform
        let lib_path = Self::get_library_path()?;

        // Load the library
        let library = unsafe {
            Library::new(&lib_path).map_err(|e| {
                LighterError::Signing(format!(
                    "Failed to load signer library at {}: {}",
                    lib_path.display(),
                    e
                ))
            })?
        };

        // Remove 0x prefix if present
        let clean_key = private_key.trim_start_matches("0x");

        let signer = Self {
            library: Arc::new(library),
            url: url.to_string(),
            private_key: clean_key.to_string(),
            chain_id: chain_id as c_int,
            api_key_index: api_key_index as c_int,
            account_index: account_index as c_int,
        };

        // Initialize the client
        signer.create_client()?;

        Ok(signer)
    }

    /// Get the library path based on the current platform
    fn get_library_path() -> Result<PathBuf> {
        let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let lib_name = if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            "signer-arm64.dylib"
        } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
            "signer-amd64.so"
        } else {
            return Err(LighterError::Signing(
                "Unsupported platform. Only macOS ARM64 and Linux AMD64 are supported".to_string(),
            ));
        };

        Ok(base_path.join("bin").join("signers").join(lib_name))
    }

    /// Initialize the client with CreateClient
    fn create_client(&self) -> Result<()> {
        unsafe {
            // Get the CreateClient function
            let create_client_fn: Symbol<
                unsafe extern "C" fn(
                    *const c_char, // url
                    *const c_char, // private_key
                    c_int,         // chain_id
                    c_int,         // api_key_index
                    c_int,         // account_index
                ) -> StrOrErr,
            > = self.library.get(b"CreateClient").map_err(|e| {
                LighterError::Signing(format!("Failed to load CreateClient function: {}", e))
            })?;

            let c_url = CString::new(self.url.as_str())
                .map_err(|e| LighterError::Signing(format!("Invalid URL string: {}", e)))?;

            let c_key = CString::new(self.private_key.as_str())
                .map_err(|e| LighterError::Signing(format!("Invalid private key string: {}", e)))?;

            // Call CreateClient
            let result = create_client_fn(
                c_url.as_ptr(),
                c_key.as_ptr(),
                self.chain_id,
                self.api_key_index,
                self.account_index,
            );

            // Check for errors
            if !result.err.is_null() {
                let error_str = CStr::from_ptr(result.err).to_string_lossy().to_string();
                // Free the error string
                libc::free(result.err as *mut libc::c_void);
                if !result.str.is_null() {
                    libc::free(result.str as *mut libc::c_void);
                }
                return Err(LighterError::Signing(format!(
                    "CreateClient failed: {}",
                    error_str
                )));
            }

            // Free the result string if it exists
            if !result.str.is_null() {
                libc::free(result.str as *mut libc::c_void);
            }

            Ok(())
        }
    }

    /// Parse the result from the library
    fn parse_result(&self, result: StrOrErr) -> Result<String> {
        unsafe {
            // Check for errors
            if !result.err.is_null() {
                let error_str = CStr::from_ptr(result.err).to_string_lossy().to_string();
                libc::free(result.err as *mut libc::c_void);
                if !result.str.is_null() {
                    libc::free(result.str as *mut libc::c_void);
                }
                return Err(LighterError::Signing(format!(
                    "Signer error: {}",
                    error_str
                )));
            }

            // Parse the value
            if result.str.is_null() {
                return Err(LighterError::Signing(
                    "Signer returned null value".to_string(),
                ));
            }

            let value_str = CStr::from_ptr(result.str).to_string_lossy().to_string();
            libc::free(result.str as *mut libc::c_void);

            Ok(value_str)
        }
    }

    /// Sign a create order request
    #[allow(clippy::too_many_arguments)]
    pub fn sign_create_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        price: i32,
        is_ask: bool,
        order_type: OrderType,
        time_in_force: TimeInForce,
        reduce_only: bool,
        trigger_price: i32,
        order_expiry: i64,
        nonce: i64,
    ) -> Result<String> {
        unsafe {
            // Get the SignCreateOrder function
            #[allow(clippy::type_complexity)]
            let sign_fn: Symbol<
                unsafe extern "C" fn(
                    c_int,      // market_index
                    c_longlong, // client_order_index
                    c_longlong, // base_amount
                    c_int,      // price
                    c_int,      // is_ask
                    c_int,      // order_type
                    c_int,      // time_in_force
                    c_int,      // reduce_only
                    c_int,      // trigger_price
                    c_longlong, // order_expiry
                    c_longlong, // nonce
                ) -> StrOrErr,
            > = self.library.get(b"SignCreateOrder").map_err(|e| {
                LighterError::Signing(format!("Failed to load SignCreateOrder function: {}", e))
            })?;

            // Convert order type and time in force to integers
            let order_type_int = match order_type {
                OrderType::Limit => 0,
                OrderType::Market => 1,
                OrderType::StopLoss => 2,
                OrderType::TakeProfit => 3,
            };

            let tif_int = match time_in_force {
                TimeInForce::Gtc => 0,
                TimeInForce::Ioc => 1,
                TimeInForce::Fok => 2,
                TimeInForce::Day => 3,
            };

            // Call the function
            let result = sign_fn(
                market_index as c_int,
                client_order_index as c_longlong,
                base_amount as c_longlong,
                price as c_int,
                if is_ask { 1 } else { 0 },
                order_type_int,
                tif_int,
                if reduce_only { 1 } else { 0 },
                trigger_price as c_int,
                order_expiry as c_longlong,
                nonce as c_longlong,
            );

            // Parse and return the result
            self.parse_result(result)
        }
    }

    /// Sign a cancel order request
    pub fn sign_cancel_order(
        &self,
        market_index: i32,
        client_cancel_index: i64,
        order_id_to_cancel: &str,
        nonce: i64,
    ) -> Result<String> {
        unsafe {
            // Get the SignCancelOrder function
            let sign_fn: Symbol<
                unsafe extern "C" fn(
                    c_int,         // market_index
                    c_longlong,    // client_cancel_index
                    *const c_char, // order_id_to_cancel
                    c_longlong,    // nonce
                ) -> StrOrErr,
            > = self.library.get(b"SignCancelOrder").map_err(|e| {
                LighterError::Signing(format!("Failed to load SignCancelOrder function: {}", e))
            })?;

            let c_order_id = CString::new(order_id_to_cancel)
                .map_err(|e| LighterError::Signing(format!("Invalid order ID string: {}", e)))?;

            // Call the function
            let result = sign_fn(
                market_index as c_int,
                client_cancel_index as c_longlong,
                c_order_id.as_ptr(),
                nonce as c_longlong,
            );

            // Parse and return the result
            self.parse_result(result)
        }
    }

    /// Sign a cancel all orders request
    pub fn sign_cancel_all_orders(
        &self,
        market_index: i32,
        client_cancel_index: i64,
        nonce: i64,
    ) -> Result<String> {
        unsafe {
            // Get the SignCancelAllOrders function
            let sign_fn: Symbol<
                unsafe extern "C" fn(
                    c_int,      // market_index
                    c_longlong, // client_cancel_index
                    c_longlong, // nonce
                ) -> StrOrErr,
            > = self.library.get(b"SignCancelAllOrders").map_err(|e| {
                LighterError::Signing(format!(
                    "Failed to load SignCancelAllOrders function: {}",
                    e
                ))
            })?;

            // Call the function
            let result = sign_fn(
                market_index as c_int,
                client_cancel_index as c_longlong,
                nonce as c_longlong,
            );

            // Parse and return the result
            self.parse_result(result)
        }
    }

    /// Sign a transfer request
    /// Note: This returns the transaction JSON. For L1 transfers, additional Ethereum signing may be needed
    pub fn sign_transfer(&self, receiver: &str, amount: i64, nonce: i64) -> Result<String> {
        unsafe {
            // Get the SignTransfer function
            let sign_fn: Symbol<
                unsafe extern "C" fn(
                    *const c_char, // receiver
                    c_longlong,    // amount
                    c_longlong,    // nonce
                ) -> StrOrErr,
            > = self.library.get(b"SignTransfer").map_err(|e| {
                LighterError::Signing(format!("Failed to load SignTransfer function: {}", e))
            })?;

            let c_receiver = CString::new(receiver)
                .map_err(|e| LighterError::Signing(format!("Invalid receiver address: {}", e)))?;

            // Call the function
            let result = sign_fn(
                c_receiver.as_ptr(),
                amount as c_longlong,
                nonce as c_longlong,
            );

            // Parse and return the result
            self.parse_result(result)
        }
    }

    /// Sign a withdraw request
    pub fn sign_withdraw(&self, receiver: &str, amount: i64, nonce: i64) -> Result<String> {
        unsafe {
            // Get the SignWithdraw function
            let sign_fn: Symbol<
                unsafe extern "C" fn(
                    *const c_char, // receiver
                    c_longlong,    // amount
                    c_longlong,    // nonce
                ) -> StrOrErr,
            > = self.library.get(b"SignWithdraw").map_err(|e| {
                LighterError::Signing(format!("Failed to load SignWithdraw function: {}", e))
            })?;

            let c_receiver = CString::new(receiver)
                .map_err(|e| LighterError::Signing(format!("Invalid receiver address: {}", e)))?;

            // Call the function
            let result = sign_fn(
                c_receiver.as_ptr(),
                amount as c_longlong,
                nonce as c_longlong,
            );

            // Parse and return the result
            self.parse_result(result)
        }
    }

    /// Switch to a different API key
    pub fn switch_api_key(&mut self, new_api_key_index: i32) -> Result<()> {
        unsafe {
            // Get the SwitchAPIKey function
            let switch_fn: Symbol<unsafe extern "C" fn(c_int) -> StrOrErr> =
                self.library.get(b"SwitchAPIKey").map_err(|e| {
                    LighterError::Signing(format!("Failed to load SwitchAPIKey function: {}", e))
                })?;

            // Call the function
            let result = switch_fn(new_api_key_index as c_int);

            // Check for errors
            if !result.err.is_null() {
                let error_str = CStr::from_ptr(result.err).to_string_lossy().to_string();
                libc::free(result.err as *mut libc::c_void);
                if !result.str.is_null() {
                    libc::free(result.str as *mut libc::c_void);
                }
                return Err(LighterError::Signing(format!(
                    "SwitchAPIKey failed: {}",
                    error_str
                )));
            }

            // Free the result string if it exists
            if !result.str.is_null() {
                libc::free(result.str as *mut libc::c_void);
            }

            // Update the internal api_key_index
            self.api_key_index = new_api_key_index as c_int;

            Ok(())
        }
    }
}

// Add libc dependency for memory management
extern "C" {
    // We use standard C free instead of a custom FreeString
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_path() {
        let path = FFISigner::get_library_path();

        if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            assert!(path.unwrap().ends_with("signer-arm64.dylib"));
        } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
            assert!(path.unwrap().ends_with("signer-amd64.so"));
        } else {
            assert!(path.is_err());
        }
    }
}
