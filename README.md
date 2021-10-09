<div id="top"></div>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![GPLv3 License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]



<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/credmp/hed">
    <img src="images/logo.png" alt="Logo" width="80" height="80">
  </a>

<h3 align="center">Host EDitor</h3>

  <p align="center">
    A command-line tool to easily manage you hosts file.
    <br />
    <br />
    <a href="https://github.com/credmp/hed">View Demo</a>
    ·
    <a href="https://github.com/credmp/hed/issues">Report Bug</a>
    ·
    <a href="https://github.com/credmp/hed/issues">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project

[![hed screencast][product-screenshot]](https://github.com/credmp/hed)

`hed` allows you to manipulate your hosts file from the command-line. By providing safe and easy commands you can add new hosts and aliases to your environment.

This tool was inspired by my students to whom I teach a Basic Cyber Security class. In this class we utilize [Hack The Box as a learning platform](https://www.youtube.com/watch?v=3b2Xul3gu_8&t=3592s) and most students struggle with editing the `hosts` file when they get started. To make this easier for them I wrote a tool that gives them a safe means of adding and removing hosts in this file.

The tool is to be used as a regular user, it will elevate privileges when it requires it by calling `sudo` and respawning the process.

<p align="right">(<a href="#top">back to top</a>)</p>



### Built With

* [Rust](https://www.rust-lang.org/)

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- GETTING STARTED -->
## Getting Started

### Installation

1. Create your local `bin` directory
   ```sh
   mkdir ~/.local/bin
   ```
2. Download the latest binary release
   ```sh
   wget https://github.com/credmp/hed/releases/latest/download/hed -O ~/.local/bin/hed
   ```
3. Make it executable
   ```sh
   chmod +x ~/.local/bin/hed
   ```
4. Ensure the `bin` directory is in your path
   ```sh
   echo export PATH=\$PATH:~/.local/bin >> ~/.zshrc # if you use zsh
   echo export PATH=\$PATH:~/.local/bin >> ~/.bashrc # if you use bash
   ```

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

[![hed screencast][product-screenshot]](https://github.com/credmp/hed)

### View the current hostsfile

`hed show` will color print the current hosts file.

```sh
hed show
```

Output:

```
#                    This is a comment
127.0.0.1	localhost	
::1	localhost	
127.0.1.1	pop-os.localdomain	pop-os
```

### Add a new entry

```sh
hed add example.com 127.1.1.1
```

Will add the following line to the hosts file.

```
127.1.1.1	example.com
```

### Add a subdomain

```sh
hed add demo.example.com
```

Will update the hosts file to add the subdomain to the parent domain as an alias

```
127.1.1.1	example.com	demo.example.com
```

### Remove a hostname

```sh
hed delete demo.example.com
```

If it is the primary `name` , the shortest alias will be chosen as new `name` for the host entry. If there are no aliases, the entire record is deleted.

```sh
hed delete 127.1.1.1
```

Will remove the entire record even if there are many aliases defined.

### Testing

Use the `--file` parameter to test the features of `hed` on a file that is not your `hosts` file.

```sh
hed --file test.txt add example.com 127.0.0.1
```

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- ROADMAP -->
## Roadmap

See the [open issues](https://github.com/credmp/hed/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- LICENSE -->
## License

Distributed under the GPLv3 License. See `LICENSE.txt` for more information.

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- CONTACT -->
## Contact

Arjen Wiersma - [@credmp](https://twitter.com/credmp) - [My website](https://www.arjenwiersma.nl/)

Project Link: [https://github.com/credmp/hed](https://github.com/credmp/hed)

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* My students for showing me that editing a `hosts` file is not that easy.

<p align="right">(<a href="#top">back to top</a>)</p>



<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[contributors-shield]: https://img.shields.io/github/contributors/credmp/hed.svg?style=for-the-badge
[contributors-url]: https://github.com/credmp/hed/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/credmp/hed.svg?style=for-the-badge
[forks-url]: https://github.com/credmp/hed/network/members
[stars-shield]: https://img.shields.io/github/stars/credmp/hed.svg?style=for-the-badge
[stars-url]: https://github.com/credmp/hed/stargazers
[issues-shield]: https://img.shields.io/github/issues/credmp/hed.svg?style=for-the-badge
[issues-url]: https://github.com/credmp/hed/issues
[license-shield]: https://img.shields.io/github/license/credmp/hed.svg?style=for-the-badge
[license-url]: https://github.com/credmp/hed/blob/master/LICENSE.txt
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=555
[linkedin-url]: https://linkedin.com/in/credmp
[product-screenshot]: images/cast.gif
