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

find_bad_clients | tee malicious_clients.new

cat malicious_clients.new malicious_clients.old malicious_clients | sort -V | uniq > malicious_clients

diff -u malicious_clients.old malicious_clients

rm -v malicious_clients.old malicious_clients.new

[[ $(id -u) == 0 ]] || { >&2 echo "Don't have root privileges, can't do anything..."; exit 1; }

for ip in $(cat malicious_clients);
do
	if iptables -C INPUT -s "${ip}" -j DROP 2>/dev/null;
	then
		echo -e " \u25cc ${ip} (already dropped)"
	else
		echo -e " \u25cb ${ip} (\e[4mNOT\e[0m already dropped; would drop)"
	fi
done

read -p "Continue? [y/N]: " confirm && [[ $confirm == [yY] || $confirm == [yY][eE][sS] ]] || exit 1

for ip in $(cat malicious_clients);
do
	if ! iptables -C INPUT -s "${ip}" -j DROP 2>/dev/null;
	then
		echo -e " \u25cb Dropping ${ip}..."

		set -x
		iptables -w -A INPUT -s "${ip}" -j DROP
		{ set +x; } 2>/dev/null
	fi
done
