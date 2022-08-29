// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;

use gitlab::api::common::NameOrId;
use gitlab::api::endpoint_prelude::*;

/// Query a single package on a project.
#[derive(Debug, Builder)]
pub struct Package<'a> {
    /// The project to query for package.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the package.
    package: u64,
}

impl<'a> Package<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> PackageBuilder<'a> {
        PackageBuilder::default()
    }
}

impl<'a> Endpoint for Package<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/packages/{}", self.project, self.package).into()
    }
}