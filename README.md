# ![muoxi_logo][logo] 
# MuOxi MUD/MU* Rustic Game Engine v0.1.0
[![Build Status][travisimg]][travislink] 

*MuOxi* is a modern library for creating [online multiplayer text
games][wikimudpage] (MU* family) in Rust using an asynchronous programming paradigm, powered by [tokio][tokio],. 
It allows developers and coders to design and flesh out their worlds in a
fast, safe, and reliable language. MuOxi engine is made available under *GPL3*. Join us on [discord][discord].


## Current Status

*Rustc 1.39>*

The codebase is currently in **Pre-Alpha**. While development continues,
the master branch is essentially a working and very minimilistic chat server 
operating over TCP. 

## Core Design Architecture

The prototype idea of how the core design is laid out into three seperate objects consisting three components
1. Staging/Proxy Server (Clients will connect to this and essentiall communicate with the engine in this stage)
2. Game Engine (all the game logic lies here and reacts to input from connected clients)
3. Database (stores information about entities, objects, and game data)

The general idea is that players will connect via Websocket to the *staging area*. In this server, clients 
are actually not connected yet to the game, unless they explicity enter. The *staging area* acts as a proxy that relays
information from players to the game itself, where then the game will react to the players input. The engine and staging area will
be seperated and communicate via a standard TCP server. The reason for this seperation, is to protect players from completely
disconnecting from the game if changes to the game engine is made.

The general layout looks like the following:

```
---------      -------------      ---------------------      -------      ---------------
| Client| <--> | Websocket | <--> |Proxy/Staging Area | <--> | TCP | <--> | Game Engine |
---------      -------------      ---------------------      -------      ---------------
```

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
[discord]: https://discord.gg/pMnBmGv
