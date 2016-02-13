# Pongo [![Build Status](https://travis-ci.org/wickus/pongo.svg?branch=master)](https://travis-ci.org/wickus/chip8)

A Pong clone written in the [Rust](http://www.rust-lang.org/) programming language. The project uses the [MIT](https://github.com/wickus/pongo/blob/master/LICENSE) license.

[![Pongo](http://wickus.github.io/pongo/images/title.png)](https://youtu.be/VgHv11kGtdQ)

[![Pongo](http://wickus.github.io/pongo/images/game.png)](https://youtu.be/VgHv11kGtdQ)

There is also a short video clip of the game on [YouTube](https://youtu.be/VgHv11kGtdQ).

## Requirements

### RUST
The game compiles against the master branch of Rust. See the Rust documentation for installation of the Rust binaries, including the Rust package manager Cargo. 

### SDL2
The game uses the cross platform media library [SDL2](https://www.libsdl.org/) for access to audio, keyboard and graphics hardware. Windows and Mac OSX binaries are available for [download](https://www.libsdl.org/download-2.0.php) from the SDL website. 

**Ubuntu**:  

```
sudo apt-get install libsdl2-dev
sudo apt-get install libsdl2-gfx-dev
sudo apt-get install libsdl2-image-dev
sudo apt-get install libsdl2-mixer-dev
sudo apt-get install libsdl2-ttf-dev
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:/usr/local/lib"
```

**HomeBrew**:  

```
brew install sdl2
brew install sdl2-gfx
brew install sdl2-image
brew install sdl2-mixer
brew install sdl2-ttf
export LIBRARY_PATH="${LIBRARY_PATH}:/usr/local/lib"
```

## How to play

```
cargo run
```
Launch the game with the above command. The title screen will show with instructions on how to play the game. The human player controls the paddle on the left while the computer controls the paddle on the right. Hit any key or click the mouse to start the game. The music will stop and the ball will immediately launch at a random angle. The goal of the game is to force the ball to hit the opposite wall. If your opponent is unable to return the ball before it hits the wall, you will gain a point. The first player to score five points wins. 

The mechanics will speed up as the game progresses. Both the ball and the computer player will start to move faster. If things get difficult, consider using one of your 'slow motion' turns. The human player is allowed three such turns, initiated by left clicking the mouse. The little green turtles at the bottom of the screen will show how many slow motion turns you have left. 

Press escape during the game to return to the title screen. Pressing escape while the title screen is showing will exit the game. Alternatively, exit the game by closing the window.

I hope you enjoy this little game. It was fun to write!

## Credits

The following fonts were obtained from [fontspace.com](fontspace.com):

* [Coffee Time](http://www.fontspace.com/weknow/coffee-time/) by WeKnow
* [DJB Pokey Dots](http://www.fontspace.com/darcy-baldwin-fonts/djb-pokey-dots/) by Darcy Baldwin
* [KG Cold Coffee](http://www.fontspace.com/kimberly-geswein/kg-cold-coffee/) by Kimberly Geswein
* [KG Happy Solid](http://www.fontspace.com/kimberly-geswein/kg-happy/) by Kimberly Geswein

The color scheme used in the game was also inspired by the DJB Pokey Dots font.

The song used during display of the title screen is called ['More Monkey Island Band'](http://soundimage.org/funny-2/more-monkey-island-band/) and is courtesy of Eric Matyas. 

## Reporting problems
If anything should go wrong, please report the issue [here](https://github.com/wickus/pongo/issues) and I will look into it. Thanks!