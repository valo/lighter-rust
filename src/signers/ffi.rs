use crate::error::{LighterError, Result};
use crate::models::common::OrderType;
use crate::models::order::TimeInForce;
use crate::signers::ethereum::EthereumSigner;
use crate::signers::ethereum::Signer as EthSigner;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_longlong};
use std::path::PathBuf;
use std::sync::Arc;

/// Result type from the Go binary
#[repr(C)]
pub struct StrOrErr {
    pub value: *mut c_char,
    pub error: *mut c_char,
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
    eth_signer: EthereumSigner,
    api_key_index: c_int,
    account_index: c_int,
}

impl FFISigner {
    /// Create a new FFI signer
    pub fn new(private_key: &str, api_key_index: i32, account_index: i32) -> Result<Self> {
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

        // Initialize the library with the private key
        Self::initialize_library(&library, private_key)?;

        // Create Ethereum signer for message signing
        let eth_signer = EthereumSigner::from_private_key(private_key)?;

        Ok(Self {
            library: Arc::new(library),
            eth_signer,
            api_key_index: api_key_index as c_int,
            account_index: account_index as c_int,
        })
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

    /// Initialize the library with the private key
    fn initialize_library(library: &Library, private_key: &str) -> Result<()> {
        unsafe {
            // Get the Init function
            let init_fn: Symbol<unsafe extern "C" fn(*const c_char, c_int, c_int) -> StrOrErr> =
                library.get(b"Init").map_err(|e| {
                    LighterError::Signing(format!("Failed to load Init function: {}", e))
                })?;

            // Remove 0x prefix if present
            let key = private_key.trim_start_matches("0x");
            let c_key = CString::new(key)
                .map_err(|e| LighterError::Signing(format!("Invalid private key string: {}", e)))?;

            // Call Init with mainnet=0 for testnet, chain_id will be set later
            let result = init_fn(c_key.as_ptr(), 0, 300); // 300 for testnet, 304 for mainnet

            // Check for errors
            if !result.error.is_null() {
                let error_str = CStr::from_ptr(result.error).to_string_lossy().to_string();
                Self::free_string(library, result.error);
                return Err(LighterError::Signing(format!("Init failed: {}", error_str)));
            }

            // Free the value if it exists
            if !result.value.is_null() {
                Self::free_string(library, result.value);
            }

            Ok(())
        }
    }

    /// Free a C string allocated by the library
    fn free_string(library: &Library, ptr: *mut c_char) {
        unsafe {
            if let Ok(free_fn) = library.get::<unsafe extern "C" fn(*mut c_char)>(b"FreeString") {
                free_fn(ptr);
            }
        }
    }

    /// Parse the result from the library
    fn parse_result(&self, result: StrOrErr) -> Result<SignedTransaction> {
        unsafe {
            // Check for errors
            if !result.error.is_null() {
                let error_str = CStr::from_ptr(result.error).to_string_lossy().to_string();
                Self::free_string(&self.library, result.error);
                if !result.value.is_null() {
                    Self::free_string(&self.library, result.value);
                }
                return Err(LighterError::Signing(format!(
                    "Signer error: {}",
                    error_str
                )));
            }

            // Parse the value
            if result.value.is_null() {
                return Err(LighterError::Signing(
                    "Signer returned null value".to_string(),
                ));
            }

            let value_str = CStr::from_ptr(result.value).to_string_lossy().to_string();
            Self::free_string(&self.library, result.value);

            // Parse as JSON
            let tx: SignedTransaction = serde_json::from_str(&value_str).map_err(|e| {
                LighterError::Signing(format!("Failed to parse signer result: {}", e))
            })?;

            Ok(tx)
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
    ) -> Result<SignedTransaction> {
        unsafe {
            // Get the SignCreateOrder function
            #[allow(clippy::type_complexity)]
            let sign_fn: Symbol<
                unsafe extern "C" fn(
                    c_int,      // api_key_index
                    c_int,      // account_index
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
                self.api_key_index,
                self.account_index,
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

            // Parse the result
            let mut tx = self.parse_result(result)?;

            // Sign the message with Ethereum signer
            let signature = self.eth_signer.sign_message(&tx.message_to_sign)?;
            tx.signature = Some(signature);

            Ok(tx)
        }
    }

    /// Sign a cancel order request
    pub fn sign_cancel_order(
        &self,
        market_index: i32,
        client_cancel_index: i64,
        order_id_to_cancel: &str,
        nonce: i64,
    ) -> Result<SignedTransaction> {
        unsafe {
            // Get the SignCancelOrder function
            let sign_fn: Symbol<
                unsafe extern "C" fn(
                    c_int,         // api_key_index
                    c_int,         // account_index
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
                self.api_key_index,
                self.account_index,
                market_index as c_int,
                client_cancel_index as c_longlong,
                c_order_id.as_ptr(),
                nonce as c_longlong,
            );

            // Parse the result
            let mut tx = self.parse_result(result)?;

            // Sign the message with Ethereum signer
            let signature = self.eth_signer.sign_message(&tx.message_to_sign)?;
            tx.signature = Some(signature);

            Ok(tx)
        }
    }

    /// Sign a cancel all orders request
    pub fn sign_cancel_all_orders(
        &self,
        market_index: i32,
        client_cancel_index: i64,
        nonce: i64,
    ) -> Result<SignedTransaction> {
        unsafe {
            // Get the SignCancelAllOrders function
            let sign_fn: Symbol<
                unsafe extern "C" fn(
                    c_int,      // api_key_index
                    c_int,      // account_index
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
                self.api_key_index,
                self.account_index,
                market_index as c_int,
                client_cancel_index as c_longlong,
                nonce as c_longlong,
            );

            // Parse the result
            let mut tx = self.parse_result(result)?;

            // Sign the message with Ethereum signer
            let signature = self.eth_signer.sign_message(&tx.message_to_sign)?;
            tx.signature = Some(signature);

            Ok(tx)
        }
    }

    /// Sign a transfer request
    pub fn sign_transfer(
        &self,
        receiver: &str,
        amount: i64,
        nonce: i64,
    ) -> Result<SignedTransaction> {
        unsafe {
            // Get the SignTransfer function
            let sign_fn: Symbol<
                unsafe extern "C" fn(
                    c_int,         // api_key_index
                    c_int,         // account_index
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
                self.api_key_index,
                self.account_index,
                c_receiver.as_ptr(),
                amount as c_longlong,
                nonce as c_longlong,
            );

            // Parse the result
            let mut tx = self.parse_result(result)?;

            // Sign the message with Ethereum signer
            let signature = self.eth_signer.sign_message(&tx.message_to_sign)?;
            tx.signature = Some(signature);

            Ok(tx)
        }
    }

    /// Sign a withdraw request
    pub fn sign_withdraw(
        &self,
        receiver: &str,
        amount: i64,
        nonce: i64,
    ) -> Result<SignedTransaction> {
        unsafe {
            // Get the SignWithdraw function
            let sign_fn: Symbol<
                unsafe extern "C" fn(
                    c_int,         // api_key_index
                    c_int,         // account_index
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
                self.api_key_index,
                self.account_index,
                c_receiver.as_ptr(),
                amount as c_longlong,
                nonce as c_longlong,
            );

            // Parse the result
            let mut tx = self.parse_result(result)?;

            // Sign the message with Ethereum signer
            let signature = self.eth_signer.sign_message(&tx.message_to_sign)?;
            tx.signature = Some(signature);

            Ok(tx)
        }
    }
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
