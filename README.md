# ![muoxi_logo][logo] 
# MuOxi MUD/MU* Rustic Game Engine v0.1.0
[![Build Status][travisimg]][travislink] 

*MuOxi* is a modern library for creating [online multiplayer text
games][wikimudpage] (MU* family) in Rust using an asynchronous programming paradigm, powered by [tokio][tokio],. 
It allows developers and coders to design and flesh out their worlds in a
fast, safe, and reliable language. MuOxi engine is made available under *GPL3*.


## Current Status

*Rustc 1.39>*

The codebase is currently in **Pre-Alpha**. While development continues,
the master branch is essentially a working and very minimilistic chat server 
operating over TCP. 


## Features and Philosophy

The MuOxi library is aimed at creating a very bare-bone library for developers
to experiment and create online text adventure games. 
As it stands the engine has the following capabilities:

* Accepts multiple connections from players
* Maintains a list of connected players
* Players can echo their input to all other connected players.


## Quick Start Guide

The following steps should get you up and running in under 10 seconds; in / directory.

* cargo run
* (using a telnet client) telnet localhost 8000


## Road Map

The bare minimum TODO features that must be implemented before I would consider it a bare mud game engine.

* Allows for new player creation
* Asks for a name and password
* saves player info (etc. name, password)
* Implements some basic commands: quit, say, tell, shutdown
* Handles players disconnecting or quitting
* Implements a periodic message every *n* seconds
* Implements some rudimentary admin control (eg. muting another player)
* Basic cardinal movement

## Future/Vision

The concept around MuOxi is not just to recreate an existing MUD game engine in Rust,
but rather to utilize the performance and practices in todays age. That being said, 
this future vision for MuOxi will change over time, but needs to fulfill some features
that I think will make this an outstanding project.

1) The Core of MuOxi will be written in Rust, expanding the core will need Rust code
2) The game logic, that handles how Mobs interact, expiermental AI integration, etc..
   will be handled in Python.
3) *add more here*






[logo]: https://github.com/duysqubix/MuOxi/blob/master/.media/cog.png
[travisimg]: https://travis-ci.org/duysqubix/MuOxi.svg?branch=master
[travislink]: https://travis-ci.org/duysqubix/MuOxi
[wikimudpage]: http://en.wikipedia.org/wiki/MUD
[tokio]: https://docs.rs/tokio/0.2.0-alpha.6/tokio/