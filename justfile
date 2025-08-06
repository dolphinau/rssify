set dotenv-load

POSTGRES_HOST := 'lwn-sub-snoozer_db'

clean:
	rm -rf target
	podman network rm lwn-sub-snoozer_network || true

init_db:
	podman network create lwn-sub-snoozer_network || true
	podman run --rm -d --replace \
	    --name {{POSTGRES_HOST}} \
		--network=lwn-sub-snoozer_network \
		-p 5432:5432 \
		--env-file .env \
		docker.io/postgres:alpine

init_app:
    podman build --tag lwn-sub-snoozer_app:latest .

init: init_db init_app

run:
    podman run --rm \
      --name lwn-sub-snoozer_app \
      --network lwn-sub-snoozer_network \
      -e POSTGRES_HOST={{POSTGRES_HOST}} \
      --env-file .env \
      --volume /tmp/lwn_sub:/rss \
      lwn-sub-snoozer_app:latest
