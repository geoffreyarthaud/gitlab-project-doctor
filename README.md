<!-- Adapted from https://github.com/othneildrew/Best-README-Template/ -->

<!-- PROJECT LOGO -->
<div align="center">

[![MIT License][license-shield]][license-url]
[![Rust][Rust]][Rust-url]
[![Forge MTE][MTE]][MTE-url]

<h3 align="center">Gitlab Project Doctor</h3>

  <p align="center">
    A CLI tool to cleanup a Gitlab repository
    <br />
</p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
    </li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->

## About The Project

Gitlab Project doctor is a CLI tool to clean up Gitlab repositories,
especially (for now) :

- Old pipelines (with jobs and jobs artifacts)
- Obsolete packages from package registry

It has first class support on Windows, macOS and Linux, with binary downloads
available
for [every release](https://github.com/geoffreyarthaud/gitlab-project-doctor/releases)
.

![Product Name Screen Shot][product-screenshot]

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->

## Getting Started

### Installation

1. Download
   the [latest release](https://github.com/geoffreyarthaud/gitlab-project-doctor/releases)
   for your OS
2. Unzip and make gitlab-project-doctor executable
   ```sh
   chmod +x gitlab-project-doctor
   ```
3. The environment variable GL_TOKEN needs to be set with a private token with
   sufficient privileges (owner of a project). For instance, on Linux :
    ```sh
    read -s GL_TOKEN # To secretly set the variable
    ```

### Usage

1. You can analyze a project from a local Git path whose first remote is a
   gitlab repository
    ```sh
    cd my_favorite_gitlab_repo
    gitlab-project-doctor .
    ```
2. Or you can analyze a project from a remote Gitlab repository
    ```sh
    gitlab-project-doctor --url https://<your-gitlab-repo.com>/your-project-path
    ```

On Gitlab, if you allow duplicate packages (same name, same version), when you
upload a package, the former one
is not deleted. gitlab-project-doctor detects :

- Generic duplicate packages: same name, same version
- Maven SNAPSHOT duplicate packages: same artifactId, same SNAPSHOT version.

### Usage on CI/CD

You can use this gitlab-project-doctor in batch mode, for example in a CI/CD job.

In a Gitlab environment, here is an example of job declaration :

```yaml
# A job in the .gitlab-ci.yml file of the project you want to clean
clean_project:
  image: $CI_REGISTRY/pub/numeco/gitlab-project-doctor:latest
  variables:
    # You need to declare a project-based private token with **owner** privilege
    GL_TOKEN: $GL_WRITE_TOKEN
  stage: build
  # The option -b activates the batch mode
  # The option -d specifies the number of days
  script:
    - gitlab-project-doctor --url $CI_PROJECT_URL -b -d 30
```
<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- ROADMAP -->

## Roadmap

- [X] Initial release with generic and maven packages detection
- [X] Internationalization (French)
- [X] Batch-mode and container image (Alpine-based)
- [ ] Container registry
- [ ] Fat git repositories

See
the [open issues](https://github.com/geoffreyarthaud/gitlab-project-doctor/issues)
for a
full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->

## Contributing

Contributions are what make the open source community such an amazing place to
learn, inspire, and create. Any contributions you make are **greatly
appreciated**.

If you have a suggestion that would make this better, please fork the repo and
create a pull request. You can also simply open an issue with the tag "
enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- LICENSE -->

## License

Distributed under the MIT License. See `LICENSE` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->

## Contact

Geoffrey Arthaud - numerique-ecologie@developpement-durable.gouv.fr

[![LinkedIn][linkedin-shield]][linkedin-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->

## Acknowledgments

* [ripgrep project](https://github.com/BurntSushi/ripgrep) for Github actions

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[license-shield]: https://img.shields.io/github/license/geoffreyarthaud/gitlab-project-doctor.svg?style=for-the-badge

[license-url]: https://github.com/geoffreyarthaud/gitlab-project-doctor/blob/master/LICENSE.txt

[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555

[linkedin-url]: https://linkedin.com/in/geoffreyarthaud

[product-screenshot]: docs_assets/gpd_screenshot.png

[Rust]: https://img.shields.io/badge/rust-000000?style=for-the-badge&logo=rust&logoColor=white

[Rust-url]: https://www.rust-lang.org/

[MTE]: https://img.shields.io/badge/forge%20MTE-0000?color=00008f&style=for-the-badge&logo=gitlab

[MTE-url]: https://gitlab-forge.din.developpement-durable.gouv.fr/pub/numeco/gitlab-project-doctor


