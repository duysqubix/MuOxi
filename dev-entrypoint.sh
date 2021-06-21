#! /bin/sh

set -e

export PATH=$PATH:$HOME/.cargo/bin

: ${APP_PATH:="$MUOXI_INSTALL_DIR"}
: ${APP_TEMP_PATH:="$APP_PATH/tmp"}
: ${APP_SETUP_LOCK:="$APP_TEMP_PATH/setup.lock"}
: ${APP_SETUP_WAIT:="5"}

lock_setup() { mkdir -p $APP_TEMP_PATH && touch $APP_SETUP_LOCK; }
unlock_setup() { rm -rf $APP_SETUP_LOCK; }
wait_setup() { echo "Waiting for app setup to finish..."; sleep $APP_SETUP_WAIT; }

trap unlock_setup HUP INT QUIT KILL TERM EXIT

if [ -z "$1" ]; then set -- cargo run --bin muoxi_staging "$@"; fi

if [ "$1" = "cargo run" ]
then

  while [ -f $APP_SETUP_LOCK ]; do wait_setup; done

  lock_setup

  cargo install

  dockerize -wait tcp://postgres:5432 -timeout 25s
  diesel migration run

  unlock_setup
fi
exec "$@"

