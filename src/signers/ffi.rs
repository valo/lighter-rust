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

#[repr(C)]
pub struct StrOrErr {
    pub str: *mut c_char,
    pub err: *mut c_char,
}

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

pub struct FFISigner {
    library: Arc<Library>,
    url: String,
    private_key: String,
    chain_id: c_int,
    api_key_index: c_int,
    account_index: c_int,
}

impl FFISigner {
    pub fn new(
        url: &str,
        private_key: &str,
        api_key_index: i32,
        account_index: i32,
    ) -> Result<Self> {
        let chain_id = if url.contains("mainnet") { 304 } else { 300 };
        let lib_path = Self::get_library_path()?;

        let library =
            unsafe { Library::new(&lib_path).map_err(|e| LighterError::Signing(e.to_string()))? };

        let clean_key = private_key.trim_start_matches("0x");

        let signer = Self {
            library: Arc::new(library),
            url: url.to_string(),
            private_key: clean_key.to_string(),
            chain_id: chain_id as c_int,
            api_key_index: api_key_index as c_int,
            account_index: account_index as c_int,
        };

        signer.create_client()?;
        Ok(signer)
    }

    fn get_library_path() -> Result<PathBuf> {
        let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        let lib_name = if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
            "signer-arm64.dylib"
        } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
            "signer-amd64.so"
        } else {
            return Err(LighterError::Signing("Unsupported platform".to_string()));
        };

        Ok(base_path.join("bin").join("signers").join(lib_name))
    }

    fn create_client(&self) -> Result<()> {
        unsafe {
            let create_client_fn: Symbol<
                unsafe extern "C" fn(*const c_char, *const c_char, c_int, c_int, c_int) -> StrOrErr,
            > = self
                .library
                .get(b"CreateClient")
                .map_err(|e| LighterError::Signing(e.to_string()))?;

            let c_url = CString::new(self.url.as_str())
                .map_err(|_| LighterError::Signing("Invalid URL".to_string()))?;
            let c_key = CString::new(self.private_key.as_str())
                .map_err(|_| LighterError::Signing("Invalid key".to_string()))?;

            let result = create_client_fn(
                c_url.as_ptr(),
                c_key.as_ptr(),
                self.chain_id,
                self.api_key_index,
                self.account_index,
            );

            if !result.err.is_null() {
                let error_str = CStr::from_ptr(result.err).to_string_lossy().to_string();
                libc::free(result.err as *mut libc::c_void);
                if !result.str.is_null() {
                    libc::free(result.str as *mut libc::c_void);
                }
                return Err(LighterError::Signing(error_str));
            }

            if !result.str.is_null() {
                libc::free(result.str as *mut libc::c_void);
            }

            Ok(())
        }
    }

    fn parse_result(&self, result: StrOrErr) -> Result<String> {
        unsafe {
            if !result.err.is_null() {
                let error_str = CStr::from_ptr(result.err).to_string_lossy().to_string();
                libc::free(result.err as *mut libc::c_void);
                if !result.str.is_null() {
                    libc::free(result.str as *mut libc::c_void);
                }
                return Err(LighterError::Signing(error_str));
            }

            if result.str.is_null() {
                return Err(LighterError::Signing("Null result".to_string()));
            }

            let value_str = CStr::from_ptr(result.str).to_string_lossy().to_string();
            libc::free(result.str as *mut libc::c_void);

            Ok(value_str)
        }
    }

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
            #[allow(clippy::type_complexity)]
            let sign_fn: Symbol<
                unsafe extern "C" fn(
                    c_int,
                    c_longlong,
                    c_longlong,
                    c_int,
                    c_int,
                    c_int,
                    c_int,
                    c_int,
                    c_int,
                    c_longlong,
                    c_longlong,
                ) -> StrOrErr,
            > = self
                .library
                .get(b"SignCreateOrder")
                .map_err(|e| LighterError::Signing(e.to_string()))?;

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

            self.parse_result(result)
        }
    }

    pub fn sign_cancel_order(
        &self,
        market_index: i32,
        client_cancel_index: i64,
        order_id_to_cancel: &str,
        nonce: i64,
    ) -> Result<String> {
        unsafe {
            let sign_fn: Symbol<
                unsafe extern "C" fn(c_int, c_longlong, *const c_char, c_longlong) -> StrOrErr,
            > = self
                .library
                .get(b"SignCancelOrder")
                .map_err(|e| LighterError::Signing(e.to_string()))?;

            let c_order_id = CString::new(order_id_to_cancel)
                .map_err(|_| LighterError::Signing("Invalid order ID".to_string()))?;

            let result = sign_fn(
                market_index as c_int,
                client_cancel_index as c_longlong,
                c_order_id.as_ptr(),
                nonce as c_longlong,
            );

            self.parse_result(result)
        }
    }

    pub fn sign_cancel_all_orders(
        &self,
        market_index: i32,
        client_cancel_index: i64,
        nonce: i64,
    ) -> Result<String> {
        unsafe {
            let sign_fn: Symbol<unsafe extern "C" fn(c_int, c_longlong, c_longlong) -> StrOrErr> =
                self.library
                    .get(b"SignCancelAllOrders")
                    .map_err(|e| LighterError::Signing(e.to_string()))?;

            let result = sign_fn(
                market_index as c_int,
                client_cancel_index as c_longlong,
                nonce as c_longlong,
            );

            self.parse_result(result)
        }
    }

    pub fn sign_transfer(&self, receiver: &str, amount: i64, nonce: i64) -> Result<String> {
        unsafe {
            let sign_fn: Symbol<
                unsafe extern "C" fn(*const c_char, c_longlong, c_longlong) -> StrOrErr,
            > = self
                .library
                .get(b"SignTransfer")
                .map_err(|e| LighterError::Signing(e.to_string()))?;

            let c_receiver = CString::new(receiver)
                .map_err(|_| LighterError::Signing("Invalid receiver".to_string()))?;

            let result = sign_fn(
                c_receiver.as_ptr(),
                amount as c_longlong,
                nonce as c_longlong,
            );

            self.parse_result(result)
        }
    }

    pub fn sign_withdraw(&self, receiver: &str, amount: i64, nonce: i64) -> Result<String> {
        unsafe {
            let sign_fn: Symbol<
                unsafe extern "C" fn(*const c_char, c_longlong, c_longlong) -> StrOrErr,
            > = self
                .library
                .get(b"SignWithdraw")
                .map_err(|e| LighterError::Signing(e.to_string()))?;

            let c_receiver = CString::new(receiver)
                .map_err(|_| LighterError::Signing("Invalid receiver".to_string()))?;

            let result = sign_fn(
                c_receiver.as_ptr(),
                amount as c_longlong,
                nonce as c_longlong,
            );

            self.parse_result(result)
        }
    }

    pub fn switch_api_key(&mut self, new_api_key_index: i32) -> Result<()> {
        unsafe {
            let switch_fn: Symbol<unsafe extern "C" fn(c_int) -> StrOrErr> = self
                .library
                .get(b"SwitchAPIKey")
                .map_err(|e| LighterError::Signing(e.to_string()))?;

            let result = switch_fn(new_api_key_index as c_int);

            if !result.err.is_null() {
                let error_str = CStr::from_ptr(result.err).to_string_lossy().to_string();
                libc::free(result.err as *mut libc::c_void);
                if !result.str.is_null() {
                    libc::free(result.str as *mut libc::c_void);
                }
                return Err(LighterError::Signing(error_str));
            }

            if !result.str.is_null() {
                libc::free(result.str as *mut libc::c_void);
            }

            self.api_key_index = new_api_key_index as c_int;
            Ok(())
        }
    }
}
