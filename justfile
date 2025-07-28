clean:
	rm -rf target
	podman network rm lwn-sub-snoozer || true

db:
	podman network create lwn-sub-snoozer || true
	podman run --name postgres --rm -p 5432:5432 --network=lwn-sub-snoozer -e POSTGRES_DB=dev -e POSTGRES_USER=root -e POSTGRES_PASSWORD=root docker.io/postgres:alpine
