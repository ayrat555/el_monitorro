#!/usr/bin/env sh

if [ -z "$SETUP_DB" ]
then
    echo "Proceeding without setting the database"
else
    echo "Setting the database"
    ./diesel database setup
fi

if [ -z "$RUN_MIGRATION" ]
then
    echo "Proceeding without running migrations"
else
    echo "Running migrations"
    ./diesel migration run
fi

case "$BOT_BINARY" in
    commands*)
        ./el_monitorro
        ;;
    sync*)
        ./sync
        ;;
    deliver*)
        ./deliver
        ;;
    cleaner*)
        ./cleaner
        ;;
    *)
        echo "Unknown binary"
        exit 1
        ;;
esac
