// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;
use gitlab::api::common::NameOrId;
use gitlab::api::endpoint_prelude::*;

/// Query a single tag on a repository from the container registry.
#[derive(Debug, Builder)]
pub struct Tag<'a> {
    /// The project to query for package.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// The ID of the repository.
    repository: u64,
    /// The name of the tag
    tag_name: String,
}

impl<'a> Tag<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> TagBuilder<'a> {
        TagBuilder::default()
    }
}

impl<'a> Endpoint for Tag<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!(
            "projects/{}/registry/repositories/{}/tags/{}",
            self.project, self.repository, self.tag_name
        )
        .into()
    }
}
