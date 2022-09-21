// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Project packages API endpoints.
//!
//! These endpoints are used for querying Gitlab container registry.
pub use self::repositories::Repositories;
pub use self::tag::Tag;

mod repositories;
mod tag;
