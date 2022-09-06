<!-- Adapted from https://github.com/othneildrew/Best-README-Template/ -->
<a name="readme-top"></a>

[![README FR][readme-fr]][readme-fr-url]
[![README EN][readme-en]][readme-en-url]

<!-- PROJECT LOGO -->
<div align="center">

[![MIT License][license-shield]][license-url]
[![Rust][Rust]][Rust-url]
[![Forge MTE][MTE]][MTE-url]

<h3 align="center">Gitlab Project Doctor</h3>

  <p align="center">
    Un outil en ligne de commande pour nettoyer un dépôt Gitlab
    <br />
</p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Sommaire</summary>
  <ol>
    <li>
      <a href="#le-projet">Le projet</a>
    </li>
    <li>
      <a href="#demarrage-rapide">Démarrage rapide</a>
    </li>
    <li><a href="#feuille-de-route">Feuille de route</a></li>
    <li><a href="#contributions">Contributions</a></li>
    <li><a href="#licence">Licence</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#remerciements">Remerciements</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->

## Le projet

Gitlab Project doctor est un outil en lignes de commande pour nettoyer un dépôt Gitlab, en particulier :

- Anciens pipelines (avec les artefacts de jobs)
- Les packages obsolètes du package registry

Il supporte Windows, MacOS, et Linux, dont les distributions binaires sont disponibles à [chaque release](https://github.com/geoffreyarthaud/gitlab-project-doctor/releases).

![Product Name Screen Shot][product-screenshot]

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->

## Démarrage rapide

### Installation

1. Télécharger la [dernière version](https://github.com/geoffreyarthaud/gitlab-project-doctor/releases)
   pour votre système d'exploitation.
2. Dézipper et rendre la commande exécutable
   ```sh
   chmod +x gitlab-project-doctor
   ```
3. La variable d'environnement GL_TOKEN a besoin d'être définie avec un token privé muni des bons
privilèges (owner du projet). Par exemple, pour Linux :
    ```sh
    read -s GL_TOKEN # Pour entrer le secret
    ```

### Usage

1. Vous pouvez analyser le projet dont le premier remote est un dépôt Gitlab
    ```sh
    cd my_favorite_gitlab_repo
    gitlab-project-doctor .
    ```
2. Ou vous pouvez analyser aà partir de l'URL du projet Gitlab
    ```sh
    gitlab-project-doctor --url https://<your-gitlab-repo.com>/your-project-path
    ```

Avec Gitlab, lorsque vous autorisez les packages dupliqués (par défaut), lorsque vous
téléversez un package avec le même nom et la même version, l'ancien package n'ets plus disponible,
mais n'est pas supprimé ! gitlab-project-doctor détecte :

- Les packages dupliqués génériques : même nom, même version
- Les anciens packages Maven SNAPSHOT : Même artifactId et même version SNAPSHOT

### Usage en CI/CD

Vous pouvez utiliser gitlab-project-doctor en mode batch, par exemple dans un job
de CI/CD.

Dans un environnement Gitlab CI, voici un exemple de déclaration :

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

## Feuille de route

- [X] Release initiale avec détection de packages generic et Maven
- [X] Internationalisation (Français)
- [X] Mode batch et image de container (basée sur Alpine)
- [ ] Container registry
- [ ] Dépôts Git trop volumineux

Cf. [open issues](https://github.com/geoffreyarthaud/gitlab-project-doctor/issues)
pour une liste de nouvelles fonctionnalités proposées et bugs connus.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->

## Contributions

Les contributions sont ce qui fait de la communauté open source un endroit incroyable pour
apprendre, s'inspirer et de créer. Toutes les contributions que vous faites sont **grandement
appréciées**.

Si vous avez une suggestion qui permettrait d'améliorer ce projet, veuillez forker le dépôt et
créer une pull request. Vous pouvez aussi simplement ouvrir une issue avec le tag "
enhancement".
N'oubliez pas de donner une étoile au projet ! Merci encore !

1. Forkez le Project
2. Créez votre branche de feature (`git checkout -b feature/AmazingFeature`)
3. Commitez votre codes (`git commit -m 'Add some AmazingFeature'`)
4. Pushez sur la branche (`git push origin feature/AmazingFeature`)
5. Ouvrez Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- LICENSE -->

## Licence

Distribué sous la licence MIT. Voir `LICENSE` pour plus d'informations.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTACT -->

## Contact

Geoffrey Arthaud - numerique-ecologie@developpement-durable.gouv.fr

[![LinkedIn][linkedin-shield]][linkedin-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->

## Remerciements

* [ripgrep project](https://github.com/BurntSushi/ripgrep) pour les Github actions

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

[readme-en]: https://img.shields.io/badge/README-%F0%9F%87%AC%F0%9F%87%A7%20English-blue.svg?style=for-the-badge

[readme-en-url]: README.md

[readme-fr]: https://img.shields.io/badge/README-%F0%9F%87%AB%F0%9F%87%B7%20Fran%C3%A7ais-blue.svg?style=for-the-badge

[readme-fr-url]: README.fr.md