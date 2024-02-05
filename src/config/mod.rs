#[cfg(feature = "tokio")]
pub mod sync;

#[cfg(not(feature = "tokio"))]
pub mod std;