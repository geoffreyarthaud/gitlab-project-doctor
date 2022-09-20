use crate::diagnosis::gitlab_connection::Project;
use crate::{fl, GitlabRepository, ReportJob, ReportPending, ReportStatus, Reportable};
use gitlab::Gitlab;
use graphql_client::*;

pub struct ConfAnalysisJob {
    pub gitlab: Gitlab,
    pub project: Project,
}

pub struct ConfAnalysisReport {
    pub gitlab: Gitlab,
    pub project: Project,
    pub report_status: Vec<ReportStatus>,
}

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "src/graphql/conf_package_duplicate.graphql",
    schema_path = "src/graphql/gitlab_schema_min.json",
    response_derives = "Debug",
    variables_derives = "Debug"
)]
struct ConfPackageDuplicateQuery;

impl Reportable for ConfAnalysisReport {
    fn report(&self) -> Vec<ReportStatus> {
        self.report_status.clone()
    }
}

impl ReportJob for ConfAnalysisJob {
    type Diagnosis = ConfAnalysisReport;

    fn diagnose(self) -> ReportPending<Self::Diagnosis> {
        ReportPending::<Self::Diagnosis> {
            pending_msg: fl!("conf-analysing"),
            job: std::thread::spawn(move || {
                let report_container = self._report_container_policy();
                let report_duplicate = self._report_duplicate_policy();
                let fix_it = !report_container.is_ok() || !report_duplicate.is_ok();
                let mut report_status = vec![report_container, report_duplicate];
                if fix_it {
                    report_status.push(ReportStatus::NA(fl!(
                        "conf-fix",
                        url = format!(
                            "{}/{}",
                            self.project.web_url, "-/settings/packages_and_registries"
                        )
                    )))
                }
                ConfAnalysisReport {
                    report_status,
                    gitlab: self.gitlab,
                    project: self.project,
                }
            }),
            progress: None,
            total: None,
        }
    }
}

impl ConfAnalysisJob {
    pub fn from(gitlab: &GitlabRepository) -> ConfAnalysisJob {
        ConfAnalysisJob {
            gitlab: gitlab.gitlab.clone(),
            project: gitlab.project.clone(),
        }
    }

    fn _report_container_policy(&self) -> ReportStatus {
        if !self.project.container_registry_enabled
            || self.project.container_expiration_policy.enabled
        {
            ReportStatus::OK(fl!("container-policy-enabled"))
        } else {
            ReportStatus::WARNING(fl!("container-policy-disabled"))
        }
    }

    fn _report_duplicate_policy(&self) -> ReportStatus {
        if let Some(policy) = self._get_duplicate_policy() {
            return if let conf_package_duplicate_query::PackagesCleanupKeepDuplicatedPackageFilesEnum::ONE_PACKAGE_FILE = policy {
                ReportStatus::OK(fl!("duplicate-assets-option-onepackage"))
            } else {
                ReportStatus::WARNING(fl!("duplicate-assets-option-warn"))
            };
        }
        ReportStatus::ERROR(fl!("duplicate-assets-option-error"))
    }

    fn _get_duplicate_policy(
        &self,
    ) -> Option<conf_package_duplicate_query::PackagesCleanupKeepDuplicatedPackageFilesEnum> {
        let variables = conf_package_duplicate_query::Variables {
            project_path: self.project.path_with_namespace.to_string(),
        };
        let query = ConfPackageDuplicateQuery::build_query(variables);
        let response = self
            .gitlab
            .graphql::<ConfPackageDuplicateQuery>(&query)
            .unwrap();
        Some(
            response
                .project?
                .packages_cleanup_policy?
                .keep_n_duplicated_package_files,
        )
    }
}
