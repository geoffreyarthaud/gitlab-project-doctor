// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;
use gitlab::api::common::NameOrId;
use gitlab::api::endpoint_prelude::*;

/// Query for registry repositories within a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Repositories<'a> {
    /// The project to query for repositories.
    #[builder(setter(into))]
    project: NameOrId<'a>,
    /// Includes an array of tags in the response.
    #[builder(default)]
    tags: Option<bool>,
    /// Includes the tags count in the response
    #[builder(default)]
    tags_count: Option<bool>,
}

impl<'a> Repositories<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> RepositoriesBuilder<'a> {
        RepositoriesBuilder::default()
    }
}

impl<'a> Endpoint for Repositories<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/registry/repositories", self.project).into()
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        params
            .push_opt("tags", self.tags)
            .push_opt("tags_count", self.tags_count);

        params
    }
}

impl<'a> Pageable for Repositories<'a> {}
