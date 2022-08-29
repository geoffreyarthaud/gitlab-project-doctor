// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Project packages API endpoints.
//!
//! These endpoints are used for querying Gitlab packages.
mod packages;
mod package;
mod package_files;

pub use self::packages::Packages;
pub use self::packages::PackageType;
pub use self::packages::PackageOrderBy;
pub use self::packages::PackageStatus;

pub use self::package::Package;

pub use self::package_files::PackageFiles;