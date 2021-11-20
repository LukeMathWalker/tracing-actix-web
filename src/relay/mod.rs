//! feature relay
//!
//! Provide convenience types
//! to automatically pipe [`RequestId`] across gRPC calls

pub mod interceptor;
pub mod middleware;
pub mod service;

use crate::RequestId;

tokio::task_local! {
  static REQUEST_ID: RequestId;
}
