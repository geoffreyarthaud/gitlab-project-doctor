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
package-no-deletion = No package has been deleted
error-insufficient-privileges = Your token has insufficient privileges
package-clean-report = Deleted {$nb_packages} packages, {$size} saved.
pipeline-analysing = Analysis of pipelines
pipeline-report = {$total_pipelines} pipelines. {$old_pipelines} pipelines are older than {$nb_days} days
pipeline-deleting = Deleting old pipelines
pipeline-clean-report = Deleted {$nb_pipelines} pipelines, {$size} saved.
pipeline-last-notdeleted = Latest pipeline is not deleted.
pipeline-no-deletion = No pipeline has been deleted
ask-delete-pipelines = Delete old pipelines ?
ask-delete-files = Delete obsolete files ?
ask-age-days = From which age in days ?
help-url = Analyze the project from the URL of Gitlab repository
help-git-path = Analyze the project from a local path of a Git repository. Ignored if url option is specified
help-batch = Batch mode : No questions, no progress bar, ideal for CI environment
help-days = Number of days from which an element is considered "old", 30 by default
help-analysis = Analysis mode : Output a detailed analysis in JSON format. No cleaning.
container-policy-enabled = The option "Clean up image tags" is enabled
container-policy-disabled = The option "Clean up image tags" is disabled.
conf-analysing = Analysis of package configuration
duplicate-assets-option-onepackage = The number of duplicate assets to keep is 1
duplicate-assets-option-warn = The number of duplicate assets to keep is NOT 1
duplicate-assets-option-error = Cannot get the number of duplicate assets to keep option
conf-fix = Fix this : {$url}
container-analysing = Analysis of container registry
container-report = {$image_count} images in container registry. {$old_image_count} are older than {$nb_days} days
container-summary = Container registry size: {$registry_size}