# Pongo [![Build Status](https://travis-ci.org/wickus/chip8.svg)](https://travis-ci.org/wickus/chip8)

[Status: Game works, but Travis build misconfigured plus Readme in draft state.]

A Pong clone written in the [Rust](http://www.rust-lang.org/) programming language. The project uses the [MIT](https://github.com/wickus/pongo/blob/master/LICENSE) license.

[![Pongo](http://wickus.github.io/pongo/images/title.png)](https://youtu.be/VgHv11kGtdQ)

[![Pongo](http://wickus.github.io/pongo/images/game.png)](https://youtu.be/VgHv11kGtdQ)

There is also a short video clip of the game on [YouTube](https://youtu.be/VgHv11kGtdQ).

## Requirements

### RUST
The game compiles against the master branch of Rust. See the section in the official Rust Book for [installing](http://doc.rust-lang.org/nightly/book/installing-rust.html) the Rust binaries, including the Rust package manager Cargo. 

### SDL2
The game uses the cross platform media library [SDL2](https://www.libsdl.org/) for access to audio, keyboard and graphics hardware. Windows and Mac OSX binaries are available for [download](https://www.libsdl.org/download-2.0.php) from the SDL website. 

**Ubuntu**:  

```
sudo apt-get install libsdl2-dev
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/usr/local/lib"
```

**HomeBrew**:  

```
brew install sdl2  
export LIBRARY_PATH="${LIBRARY_PATH}:/usr/local/lib"
```

## How to play

```
cargo run
```

## Reporting problems
If anything should go wrong, please report the issue [here](https://github.com/wickus/pongo/issues) and I will look into it. Thanks!