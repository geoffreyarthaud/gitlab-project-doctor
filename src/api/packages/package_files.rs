// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;
use gitlab::api::common::NameOrId;
use gitlab::api::endpoint_prelude::*;

/// Query for files within a package.
#[derive(Debug, Builder)]
pub struct PackageFiles<'a> {
    /// The project to query for the package.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the package.
    package: u64,
}

impl<'a> PackageFiles<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> PackageFilesBuilder<'a> {
        PackageFilesBuilder::default()
    }
}

impl<'a> Endpoint for PackageFiles<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/packages/{}/package_files",
            self.project, self.package
        )
        .into()
    }
}

impl<'a> Pageable for PackageFiles<'a> {}
