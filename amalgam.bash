#!/bin/bash

cp malicious_clients malicious_clients.old

function find_bad_clients() (
	set -x

	log_tmp="$(mktemp)"

	journalctl -u sshd > "$log_tmp"

	function find_offenders {
		cat "$log_tmp" \
			| rg -i "failed password|invalid user|connection closed by" \
			| rg -i '([\da-f\.:]+) port \d+' -or '$1' \
			| sort -V \
			| uniq
	}

	function find_accepts {
		cat "$log_tmp" \
			| rg -i 'accepted' \
			| rg -i '([\da-f\.:]+) port \d+' -or '$1' \
			| sort -V \
			| uniq
	}

	offenders="$(mktemp)"
	find_offenders > "$offenders"

	accepts="$(mktemp)"
	find_accepts > "$accepts"

	for offender in `cat "$offenders"`;
	do
		if ! rg -qi "$offender" -- "$accepts";
		then
			#>&2 echo "$offender tried to log in, not found in our acceptance list; probably bad, blocking"
			echo "$offender"
		fi
	done

	rm -rf "$log_tmp" "$offenders" "$accepts"
)

function generate_new_iptables_commands() (
	set -x

	iptables_dumploc="$(mktemp)"

	# Dump iptables to a file so we don't break the thing
	sudo iptables -n --list | sudo tee "$iptables_dumploc" >/dev/null

	echo "set -x"

	# Malicious clients should already be generated
	for ip in $(cat malicious_clients);
	do
		# If we haven't cleaned that bad client out, give me the commands to run
		if ! rg -q "$ip" -- "$iptables_dumploc";
		then
			echo "sudo iptables -A INPUT -s $ip -j DROP"
		fi
	done

	sudo rm -f "$iptables_dumploc"
)

find_bad_clients | tee malicious_clients.new

cat malicious_clients.new malicious_clients.old malicious_clients | sort -V | uniq > malicious_clients

diff -u malicious_clients.old malicious_clients

rm -v malicious_clients.old malicious_clients.new

generate_new_iptables_commands > bombs_away.sh
