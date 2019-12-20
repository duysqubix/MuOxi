# ![muoxi_logo][logo] 
# MuOxi MUD/MU* Rustic Game Engine v0.1.0
[![Build Status][travisimg]][travislink] 

*MuOxi* is a modern library for creating [online multiplayer text
games][wikimudpage] (MU* family) in Rust using the powerful and flexible [Amethyst][amethyst] game engine,. 
It allows developers and coders to design and flesh out their worlds in a
fast, safe, and reliable language. MuOxi engine is made available under *GPL3*. Join us on [discord][discord].


## Current Status

*Rustc 1.39> [Stable/Nightly]*

The codebase is currently in *alpha* stage . While development continues,
the master branch is essentially a working and very minimilistic weboscket server 
operating over TCP. 

## Road Map

The bare minimum TODO features that must be implemented before I would consider it a bare mud game engine.

* Allows for multiple communication protocols (*telnet, MCCP, websocket, etc*)
* Allows for new player creation
* Asks for a name and password
* saves player info (etc. name, password)
* Implements some basic commands: quit, say, tell, shutdown
* Handles players disconnecting or quitting
* Implements a periodic message every *n* seconds
* Implements some rudimentary admin control (eg. muting another player)
* Basic cardinal movement

## Core Design Architecture

The prototype idea of how the core design is laid out into three seperate objects.
1. Staging/Proxy Server *(Clients will connect to this server and essentially communicate with the engine via here stage)*
2. Game Engine *(all the game logic lies here and reacts to input from connected clients)*
3. Database *(stores information about entities, objects, and game data)* 

The idea is that players will connect via Websocket to the *proxy server*. In this server, clients 
are not actually connected to the game, unless they explicity enter. The *staging area* acts as a proxy that relays
information from players to the game itself, where then the game will react to the players input. The engine and staging area will
be seperated and communicate via a standard TCP server. The reason for this seperation, is to protect players from completely
disconnecting from the game if changes to the game engine is made.

The support for multiple type of connections is a must. Therefore the following shows an example design layout that
has the ability to handle multiple communication protocols. Each comm type will have a unique port that must be addressed
and acts like a proxy to the main Staging Area.

```
------------
| Websocket | <---------------- \
------------                     \
----------                        ---------------------             ---------------
| Telnet | ---------------------->|Proxy/Staging Area | <-- TCP --> | Game Engine |
----------                        ---------------------             ---------------
                                 /
--------                        /
| MCCP | <----------------------
--------
```

This design is still in prototype phase.

## Features and Philosophy

The MuOxi library is aimed at creating a very simplistic and robust library for developers
to experiment and create online text adventure games. 
As it stands the engine has the following capabilities:

* Accepts multiple connections from players
* Maintains a list of connected players
* Players can echo their input to all other connected players.


## Quick Start Guide

The project contains two seperate bin that can both be evoked from the command line:

* cargo run --bin muoxi_websocket
    * Starts the websocket server listening for incoming webclients, *default 8000*

* cargo run --bin muoxi_telnet
    * starts the basic telnet server listening for incoming telnet clients, *default 8001*

* *(Not implemented Yet)* cargo run --bin muoxi_proxy
    * starts the main Proxy Staging server where all clients will *live*, this area is where clients will communicate to the game engine.

* *(Not Implemented Yet)* cargo run --bin muoxi_engine
    * Starts the main game engine running in it's own seperate process. The whole game is contained
    within a TCP listening server that exchanges information back and forth between to the Proxy Server




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
[amethyst]: https://amethyst.rs/
[discord]: https://discord.gg/pMnBmGv
