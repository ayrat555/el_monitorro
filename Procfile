#release: ./target/release/diesel migration run
release: ./target/release/diesel database setup

### Uncomment this to use with Heroku paid tier
#worker: ./target/release/el_monitorro
#cleaner: ./target/release/cleaner
#syncer: ./target/release/sync
#deliver: ./target/release/deliver

### Uncomment this to use with Heroku free tier (run 2 max at the same time)
worker: ./target/release/el_monitorro & ./target/release/deliver
cleaner: ./target/release/cleaner
syncer: ./target/release/sync
