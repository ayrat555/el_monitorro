#!/usr/bin/env sh


if [ -z "$RUN_MIGRATION" ]
then
    echo "Proceeding without setting the database"
else
    echo "Setting the database"
    diesel database setup
fi


case "$BOT_BINARY" in
    commands*)
        ./target/release/el_monitorro
        ;;
    sync*)
        ./target/release/sync
        ;;
    deliver*)
        ./target/release/deliver
        ;;
    cleaner*)
        ./target/release/cleaner
        ;;
    *)
        echo "Unknown binary"
        exit 1
        ;;
esac
