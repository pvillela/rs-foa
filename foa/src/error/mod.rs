//===========================
// region:      --- modules

mod app_error;
mod box_error;
mod core_error;
mod deser;
mod foa_error;
mod full_kind;
mod misc;
mod payload;
mod static_str;
mod tags;
mod utils;

// endregion:   --- modules

//===========================
// region:      --- flattened

pub use app_error::*;
pub use box_error::*;
pub use core_error::*;
pub use deser::*;
pub use foa_error::*;
pub use full_kind::*;
pub use misc::*;
pub use payload::*;
pub use tags::*;
pub use utils::*;

// endregion:   --- flattened
