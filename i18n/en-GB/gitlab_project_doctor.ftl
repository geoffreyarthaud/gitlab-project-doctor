connecting-to-gitlab = Connecting to Gitlab
gitlab-repo = Gitlab repository : {$repo}
error-gl-token = GL_TOKEN environment variable must contain a valid Gitlab private token
error-not-gitlab-repo = This URL is not a gitlab repository
error-not-git-repo = This dir is not a Git repository
error-no-gitlab-remote = This dir does not contain a gitlab remote
size-storage = Storage size:
size-git-repo = Git repository size:
size-artifacts = Artifact jobs size:
size-packages = Package registry size:
package-analysing = Analysis of packages
no-cicd = No CI/CD configured for this project
error = Error:
package-report = {$nb_packages} packages. {$nb_files} files are obsolete ({$size})
package-deleting = Deleting obsolete packages files
error-insufficient-privileges = Your token has insufficient privileges
package-clean-report = Deleted {$nb_packages} packages, {$size} saved.
pipeline-analysing = Analysis of pipelines
pipeline-report = {$total_pipelines} pipelines. {$old_pipelines} pipelines are older than {$nb_days} days
pipeline-deleting = Deleting old pipelines
pipeline-clean-report = Deleted {$nb_pipelines} pipelines, {$size} saved.
pipeline-last-notdeleted = Latest pipeline is not deleted.
ask-delete-pipelines = Delete old pipelines ?
ask-delete-files = Delete obsolete files ?
ask-age-days = From which age in days ?
help-url = Analyze the project from the URL of Gitlab repository
help-git-path = Analyze the project from a local path of a Git repository. Ignored if url option is specified
help-batch = Batch mode : No questions, no progress bar, ideal for CI environment
help-days = Number of days from which an element is considered "old", 30 by default