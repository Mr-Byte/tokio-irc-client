#!/bin/bash

set -o pipefail
set -eu
set -x

DOCKER_IRCD_NAME="tokio_test_ircd"

tmpdir=$(mktemp -d)
docker rm -f ${DOCKER_IRCD_NAME} 2>/dev/null || true
docker run --net=host -d --name ${DOCKER_IRCD_NAME} -p 6667:6667 inspircd/inspircd-docker:2.0.24
trap "rm -rf ${tmpdir}; docker rm -f ${DOCKER_IRCD_NAME}" EXIT

for i in $(seq 1 10); do
	if nc localhost 6667 -w 1 -q 1 </dev/null; then
		break
	fi
	if [[ "$i" == 10 ]]; then
		echo "irc did not come up in time"
		exit 1
	fi
	sleep 2
done

export IRC_SERVER="localhost:6667"

./target/debug/examples/print_messages > "$tmpdir/recvd" &
print_messages_pid=$!
trap "kill -9 $print_messages_pid; rm -rf ${tmpdir}; docker rm -f ${DOCKER_IRCD_NAME}" EXIT

# Give it time to join channel
sleep 5

./target/debug/examples/send_message

expected=$(cat <<EOF
<RustBot2> Hello World!
<RustBot2> Goodbye world
EOF
)

if [[ "${expected}" != "$(cat "${tmpdir}/recvd")" ]]; then
	echo "Expected contents to contain '${expected}'; was:"
	cat "$tmpdir/recvd"
	exit 1
fi
