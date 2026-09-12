//! Roche Accu-Chek USB discovery, IEEE 11073 session handling, and measurement parsing.
//!
//! The state machine is intentionally read-only. It never changes meter time or settings.
//! Behaviour is cross-checked against Tidepool's maintained Roche driver and the independent
//! libusb implementation by emogenet.

use std::time::Duration;

use log::{info, warn};
use rusb::{Context, DeviceHandle, UsbContext};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::AccuChekError;
use crate::protocol::*;

include!("device_parts/types.rs");
include!("device_parts/messages.rs");
include!("device_parts/session.rs");
include!("device_parts/tests.rs");
