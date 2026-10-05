CREDITS, LICENSES AND INSPIRATIONS
=================================

Omnifetch is built with respect for the open-source community and the CLI
toolchain ecosystem. Below are the projects, authors and licenses whose ideas,
artwork and design philosophies contributed to it.


1. ASCII ART AND DISTRO LOGOS
-----------------------------

Fastfetch
  https://github.com/fastfetch-cli/fastfetch
  Author:  Linus Dierheimer and Fastfetch contributors
  What:    the library of 550+ full-size and mini distribution ASCII logos in
           assets/normal and assets/mini, sourced and adapted from Fastfetch
  License: MIT

    MIT License

    Copyright (c) 2021-2026 Linus Dierheimer and Fastfetch contributors

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in
    all copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.


2. IDEAS AND OUTPUT FORMATS
--------------------------

The design of each fetch tool below informed module selection, ordering and
layout. No code was copied.

Onefetch      https://github.com/o2sh/onefetch            git module, Onefetch style
Cpufetch      https://github.com/robinhargreaves/cpufetch  cpu rendering
Hyfetch       https://github.com/andreansaraiva/hyfetch
Nitch         https://github.com/nicolatos/nitch
Catnap        https://github.com/lvyaoyu/catnap
Macchina      https://github.com/SeptemberFoxworth/macchina
Pfetch        https://github.com/dvander/pub
Neofetch      https://github.com/dylanaraps/neofetch
Sysprint      https://github.com/antonionet             sysprint layout preset
Paleofetch    https://github.com/otreblan/paleofetch


3. RUST CRATES
--------------

Omnifetch is built with minimal external dependencies. Its only external runtime dependency is:

  libc                 system calls and process ids


4. LOGOS AND ARTWORK
--------------------

Distribution logos are plain text files under assets/normal and assets/mini.
Any that came from Fastfetch carry its MIT notice above. Logos added
independently are simple typographic renderings of distribution names and carry
no third-party rights.


5. LICENCE OF OMNIFETCH ITSELF
------------------------------

MIT. See the licence field in Cargo.toml.