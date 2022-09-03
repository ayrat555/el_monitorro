db:
	docker run --rm -d --name postgres -p 5432:5432 \
  -e POSTGRES_DB=el_monitorro \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  postgres:latest
clippy:
	cargo clippy --all-features
diesel:
	DATABASE_URL=postgres://postgres:postgres@localhost/el_monitorro diesel migration run
stop:
	docker kill postgres
tests:
	DATABASE_URL=postgres://postgres:postgres@localhost/el_monitorro cargo test --all-features -- --color always --nocapture

ignored:
	DATABASE_URL=postgres://postgres:postgres@localhost/el_monitorro cargo test --all-features -- --color always --nocapture --ignored

doc:
	cargo doc --open
