// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use derive_builder::Builder;
use gitlab::api::common::{NameOrId, SortOrder};
use gitlab::api::endpoint_prelude::*;
use gitlab::api::ParamValue;

/// Keys packages results may be ordered by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageOrderBy {
    /// When the pipeline was last updated.
    CreatedAt,
    /// Order by name of the package.
    Name,
    /// Order by version of the package.
    Version,
    /// Order by package type.
    Type,
}

impl PackageOrderBy {
    /// The ordering as a query parameter.
    fn as_str(self) -> &'static str {
        match self {
            PackageOrderBy::CreatedAt => "created_at",
            PackageOrderBy::Name => "name",
            PackageOrderBy::Version => "version",
            PackageOrderBy::Type => "type",
        }
    }
}

impl ParamValue<'static> for PackageOrderBy {
    fn as_value(&self) -> Cow<'static, str> {
        self.as_str().into()
    }
}

/// The type of a package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageType {
    /// Conan package type.
    Conan,
    /// Maven package type.
    Maven,
    /// Npm package type.
    Npm,
    /// Pypi package type.
    Pypi,
    /// Composer package type.
    Composer,
    /// Nuget package type.
    Nuget,
    /// Helm package type.
    Helm,
    /// Terraform module package type.
    TerraformModule,
    /// Golang package type.
    Golang,
}

impl PackageType {
    /// The package type as a query parameter.
    fn as_str(self) -> &'static str {
        match self {
            PackageType::Conan => "conan",
            PackageType::Maven => "maven",
            PackageType::Npm => "npm",
            PackageType::Pypi => "pypi",
            PackageType::Composer => "composer",
            PackageType::Nuget => "nuget",
            PackageType::Helm => "helm",
            PackageType::TerraformModule => "terraform_module",
            PackageType::Golang => "golang",
        }
    }
}

impl ParamValue<'static> for PackageType {
    fn as_value(&self) -> Cow<'static, str> {
        self.as_str().into()
    }
}

/// The status of a package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageStatus {
    /// The default package status
    Default,
    /// The status of a hidden package
    Hidden,
    /// The processiog package status
    Processing,
    /// The status of a package in error
    Error,
    /// The status of a pending package for destruction
    PendingDestruction,
}

impl PackageStatus {
    /// The package type as a query parameter.
    fn as_str(self) -> &'static str {
        match self {
            PackageStatus::Default => "default",
            PackageStatus::Hidden => "hidden",
            PackageStatus::Processing => "processing",
            PackageStatus::Error => "error",
            PackageStatus::PendingDestruction => "pending_destruction",
        }
    }
}

impl ParamValue<'static> for PackageStatus {
    fn as_value(&self) -> Cow<'static, str> {
        self.as_str().into()
    }
}

/// Query for pipelines within a project.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Packages<'a> {
    /// The project to query for packages.
    #[builder(setter(into))]
    project: NameOrId<'a>,

    /// Order results by a given key.
    #[builder(default)]
    order_by: Option<PackageOrderBy>,
    /// Sort order for resulting packages.
    #[builder(default)]
    sort: Option<SortOrder>,
    /// Filter packages by its type.
    #[builder(default)]
    package_type: Option<PackageType>,
    /// Filter packages by its name.
    #[builder(setter(into), default)]
    package_name: Option<Cow<'a, str>>,
    /// Filter packages with or without versionless packages.
    #[builder(default)]
    include_versionless: Option<bool>,
    /// Filter packages by its status.
    #[builder(default)]
    status: Option<PackageStatus>,
}

impl<'a> Packages<'a> {
    /// Create a builder for the endpoint.
    pub fn builder() -> PackagesBuilder<'a> {
        PackagesBuilder::default()
    }
}

impl<'a> Endpoint for Packages<'a> {
    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Cow<'static, str> {
        format!("projects/{}/packages", self.project).into()
    }

    fn parameters(&self) -> QueryParams {
        let mut params = QueryParams::default();

        params
            .push_opt("order_by", self.order_by)
            .push_opt("sort", self.sort)
            .push_opt("package_type", self.package_type)
            .push_opt("package_name", self.package_name.as_ref())
            .push_opt("include_versionless", self.include_versionless)
            .push_opt("status", self.status);

        params
    }
}

impl<'a> Pageable for Packages<'a> {}
