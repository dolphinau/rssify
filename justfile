set dotenv-load

POSTGRES_HOST := 'rssify_db'
TEMP_DIR := `mktemp -d`

clean:
	rm -rf target
	podman stop rssify_db || true
	podman network rm rssify_network || true

init_db:
	podman network create rssify_network || true
	podman run --rm -d --replace \
	    --name {{POSTGRES_HOST}} \
		--network=rssify_network \
		-p 5432:5432 \
		--env-file .env \
		docker.io/postgres:alpine

build:
    podman build --tag rssify_app:latest .

init: init_db build

attach:
    podman run -it --rm \
      --name rssify_app \
      --network rssify_network \
      -e POSTGRES_HOST={{POSTGRES_HOST}} \
      --env-file .env \
      --volume {{TEMP_DIR}}:/rss \
      rssify_app:latest sh

run:
    podman run --rm \
      --name rssify_app \
      --network rssify_network \
      -e POSTGRES_HOST={{POSTGRES_HOST}} \
      --env-file .env \
      --volume {{TEMP_DIR}}:/rss \
      rssify_app:latest
