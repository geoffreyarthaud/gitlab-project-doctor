// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use gitlab::api::common::NameOrId;
use gitlab::api::endpoint_prelude::*;

/// Delete a package.
#[derive(Debug, Builder)]
pub struct DeletePackageFile<'a> {
    /// The project to delete the package from.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the package.
    package: u64,
    /// The ID of the file.
    file: u64
}

impl<'a> DeletePackageFile<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> DeletePackageFileBuilder<'a> {
        DeletePackageFileBuilder::default()
    }
}

impl<'a> Endpoint for DeletePackageFile<'a> {
    fn method(&self) -> Method {
        Method::DELETE
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/packages/{}/package_files/{}", self.project, self.package, self.file)
            .into()
    }
}

