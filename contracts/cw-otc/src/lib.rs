#[cfg(not(feature = "library"))]
pub mod contract;
mod execute;
mod functions;
mod query;
mod response;
mod state;
