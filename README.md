# ![muoxi_logo][logo] 
# MuOxi MUD/MU* Rustic Game Engine v0.1.0
[![Build Status][travisimg]][travislink] [![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0) 



*MuOxi* is a modern library for creating [online multiplayer text
games][wikimudpage] (MU* family) using the powerful features offered by Rust; backed by [Tokio][tokio] and [Diesel][diesel],. 
It allows developers and coders to design and flesh out their worlds in a
fast, safe, and reliable language. Explore MuOxi API the *[rustacean][gh-pages-site]* way Join us on [discord][discord].


## Current Status

*For Nightly Builds* an update on Rust nightly seems to have broken something with the `diesel` crate. 

The codebase is currently in *alpha* stage . Majority of development is done on the `master` 
branch. There is a working TCP server that allows
for multiple connections and handles them accordingly. Effort is focused at the moment in 
designing the database architecture utilizing [Diesel][diesel] with [PostgreSQL][postgresql] backend.

## Contributions

Please submit PR's for approval and discussions.
No matter your skill level any sort of effort into this project is extremely welcomed. For those wanting to contribute, checkout the `master` branch
and submit PR's. Any questions or information, we welcome you at our [discord][discord] server. Come on by.

## Road Map

The bare minimum TODO features that must be implemented before I would consider releasing v0.1.1

* Allows for multiple communication protocols (*telnet, MCCP, websocket, etc*)
* Allows for new player creation
* Asks for a name and password
* saves player info (etc. name, password)
* Implements some basic commands: quit, say, tell, shutdown
* ~~Handles players disconnecting or quitting~~
* Implements a periodic message every *n* seconds
* Implements some rudimentary admin control (eg. muting another player)
* Basic cardinal movement
* ~~Implements a backend database, with friendly API tailored to MuOxi~~
* Simple game showcasing features of MuOxi

## Getting Started

In order for MuOxi applications to work as expected, it is necessary to have a fully working PostgreSQL and Redis server running.
You can change it in the code to what user postgres should be using, but the default is `muoxi` with password `muoxi`. Redis server
should be running as well, if you have successfully installed in on your OS, you can start it from the terminal using `redis-server`.
In the future I will add scripts that will do this for you, upon initiliazing MuOxi. For now, you must unfortuantly do everything by hand.
Hopefully the following steps will guide you through the process to set up your environment for working on the code base.


The following does assume you are working on Linux based OS. If you are using Windows >=10, use WSL as a linux sub and for everything else: Cygwin. However, I haven't
tested this on a pure Windows environment..

### Install some misc things, that have prevented me from compiling all the necessary Rust libraries.

1. `sudo apt update && upgrade`
2. `sudo apt install libpq-dev`
3. Attempt `cargo build`, everything should build fine now..

### Set Up Redis Server

First you must install redis:

1.  `sudo apt install redis-server -y`
2.  `sudo service redis-server start` # enable on startup
3.  To enable to make sure it is running you can manually start it with `redis-server` 

To check if redis has installed and is running successfully run: `redis-cli` in the cli you should be greeted with `<127.0.0.1:6379>


### Set Up Postgres SQL for the storage

1. sudo apt install postgresql
2. sudo service postgresql start
3. sudo su - postgres
4. createuser --superuser muoxi
5. psql
6. \password muoxi (muoxi for password)
7. \q (to exit)
8. createdb muoxi
9. exit and now everything should be set up

### Install Diesel Cli for migrations and database management

Diesel is the Rust go-to solution for abstraction over database manipulation. It allows Rust code to be natively wrapped around the drivers for different SQL-based databases. This is equivalent to something like Django or Twistd for the python lovers.

1. cargo install diesel_cli --no-default-features --postgres
2. diesel migration run

That should be the end of basic setup - you can test the connection by running `cargo run --bin muoxi\_staging` and pointing any telnet client to: `127.0.0.1:8000`. You should be greeted by the MuOxi logo.
Have fun :)

## Quick Start Guide

The project contains multiple  bins that can be evoked from the command line:

* *(Not working as intended at the moment)* **cargo run --bin muoxi_web**
    * Starts the websocket server listening for incoming webclients, *default 8001*

* **cargo run --bin muoxi_staging**
    * starts the main Proxy Staging server where all clients will *live*, this area is where clients will communicate to the game engine. Direct telnet clients can connect this is server via port *8000*

* **cargo run --bin muoxi_watchdog**
  * starts the external process that monitors changes to configuration json files. Once a change has been detected it triggers an update protocol to update MongoDB

* **cargo run --bin muoxi_engine**
    * Starts the main game engine running in it's own seperate process. The whole game is contained
    within a TCP listening server that exchanges information back and forth between to the Proxy Server. *Right now it is just an echo server*



## Database Design Architecture

The database design is seperated into four different layers, with different levels of abstraction.
MuOxi utilizes a [PostgreSQL][postgresql] backend for its storage needs and [Redis][redis] for its caching and fast retrieval needs.
 A unique design approach has been taken that allows information 
to be kept safe from database corruption, brownouts, or blackouts. The ideology is
as follows:

```
 Layer 1: JSON Files <--------
              |              |
             \ /             |
 Layer 2:  Postgres ------   |
                         |   |
                         |   |
-----------------------  |   |
|Layer 3: Cache/Memory|  |   |
-----------------------  |   |
        |    / \         |   |
       \ /    |         \ /  |
 Layer 4: MuOxi Applications--
```

#### Layer 1: Flat Files

The entire database actually lives in JSON files from accounts, mobs, players, equipment, spells, skills, etc... 
JSON files where chosen because of the *hyper-fast* libraries available for manipulating json files in Rust and its friendly human readability.
A seperate process called the *watchdog* monitors custom defined `.json` files in the 
`/config` directory for any changes to contents themselves. Upon a detected change it triggers an upload piece of logic
that *updates* [postgreSQL][postgresql] database using [Diesel][diesel], which leads us to layer 2 of the design.


#### Layer 2: PostgreSQL

This is where all persistent data will live throughout, and past, the life-span of MuOxi. Powered by an ORM management system, [Diesel][diesel] with
[postgreSQL][postgresql] backend. The database should always be a reflection of what is stored in the `.json` files. MuOxi applications 
queries straight from the database. 

#### Layer 3: Cache/In-Memory 

This is a helper layer that is dominated by [Redis][redis], for quick retrieval of information and adding ad-hoc non-persistent data such as combat,
triggers, and other various information that would not be detrimental if a shutdown occured for whatever reason. This layer is meant to be used on a
*use-if-needed* basis.

#### Layer 4: MuOxi Applications

This is the layer where MuOxi will actually use all persistent and non-persistent data to drive the actual engine itself. Whether it be
handling different states of connected clients, combat data, player information, and any-and-all other memory will be read from the cached database
to keep the engine running. Upon an action within MuOxi that would causes a change to the Database, MuOxi will actually write to the flat-files
instead of directly to PostgreSQL. This was a throughouly thought out process to keep PostgreSQL a read-only database, from the perspective of the engine itself.
When a change occurs and MuOxi writes to the flat files we began again at layer 1 of the design. __It is the responsibility of the WatchDog to monitor changes to
the json files and update PostgresSQL. PostgresSQL and the JSON files should always be a reflection of each other.__

## Core Design Architecture

The prototype idea of how the core design is laid out into three seperate objects.
1. Staging/Proxy Server *(Clients will connect to this server and essentially communicate with the engine via this stage)*
2. Game Engine *(all the game logic lies here and reacts to input from connected clients)*
3. Database *(stores information about entities, objects, and game data)* 
4. Communication *( Each supported comm client (MCCP, telnet, websocket) will act as a full-duplex proxy that communicates with the Staging Server)*

The idea is that players will connect via one of the supported communication protocols to the *proxy server*. In this server, clients 
are not actually connected to the game, unless they explicity enter. The *staging area* holds all connected client information such as 
player accounts, different characters for each player, and general settings. When a client acutally connects to the game itself
the server acts as a proxy that relays information from players to the game engine, where the engine will then react to the players input. 
The engine and staging area will be seperated and communicate via a standard TCP server. The reason for this seperation, is to protect players from completely
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
* Hold shared states between connected clients
* Removes clients upon disconnection



## Future/Vision

The concept around MuOxi is not just to recreate an existing MUD game engine in Rust,
but rather to utilize the performance and safety that Rust has to offer. That being said, 
this future vision for MuOxi will change over time, but it needs to fulfill some features
that I think will make this an outstanding project.

1) The Core of MuOxi will be written in Rust, expanding the core will need Rust code
2) The game logic, that handles how Mobs interact, expiermental mob AI integration, etc..
   will be handled in Python.
3) *add more here*






[logo]: https://github.com/duysqubix/MuOxi/blob/master/.media/cog.png
[travisimg]: https://travis-ci.org/duysqubix/MuOxi.svg?branch=master
[travislink]: https://travis-ci.org/duysqubix/MuOxi
[wikimudpage]: http://en.wikipedia.org/wiki/MUD
[amethyst]: https://amethyst.rs/
[discord]: https://discord.gg/H6Sh3CJ
[tokio]: https://github.com/tokio-rs/tokio
[diesel]: http://diesel.rs/
[bson]: http://bsonspec.org/
[redis]: https://redis.io/
[gh-pages-site]: https://duysqubix.github.io/MuOxi/
[postgresql]: https://www.postgresql.org/
