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

function is_ipv4() (
	echo "$1" | rg -q '^[\d\.]+$'
)

function is_ipv4_ipset() (
	sudo ipset list -t "$1" | rg -q -U --multiline-dotall 'Name: (.*?)$.*?^Header: .*? \binet\b'
)

function is_ipv6() (
	echo "$1" | rg -q '^[\da-fA-F:]$'
)

function is_ipv6_ipset() (
	sudo ipset list -t "$1" | rg -q -U --multiline-dotall 'Name: (.*?)$.*?^Header: .*? \binet6\b'
)

function ipset_contains_ip() (
	is_ipv4 "$2" && is_ipv4_ipset "$ipset" && { set -x; ipset test "$1" "$2"; { set +x; } 2>/dev/null; } && exit "$?"
	is_ipv6 "$2" && is_ipv6_ipset "$ipset" && { set -x; ipset test "$1" "$2"; { set +x; } 2>/dev/null; } && exit "$?"
	exit 1
)

function ipsets_block_ip() (
	ipsets=`iptables -w -nL | rg -or '$1' '^DROP .* match-set \b(\w*)\b src'`
	for ipset in $ipsets;
	do
		{ ipset_contains_ip "$ipset" "$1" 2>&1 | rg -q 'is in'; } && exit 0
	done
	exit 1
)


function ipsets_block_ip6() (
	ipsets=`ip6tables -w -nL | rg -or '$1' '^DROP .* match-set \b(\w*)\b src'`
	for ipset in "${ipsets}";
	do
		{ ipset_contains_ip "$ipset" "$1" 2>&1 | rg -q 'is in'; } && exit 0
	done
	exit 1
)

function ip_is_blocked() (
	iptables -C INPUT -s "$1" -j DROP 2>/dev/null || ipsets_block_ip "$1"
)

function ip6_is_blocked() (
	ip6tables -C INPUT -s "$1" -j DROP 2>/dev/null || ipsets_block_ip6 "$1"
)

find_bad_clients | tee malicious_clients.new

cat malicious_clients.new malicious_clients.old malicious_clients | sort -V | uniq > malicious_clients

diff -u malicious_clients.old malicious_clients

rm -v malicious_clients.old malicious_clients.new

[[ $(id -u) == 0 ]] || { >&2 echo "Don't have root privileges, can't do anything..."; exit 1; }

for ip in $(cat malicious_clients);
do
	if echo "${ip}" | rg -q '^[\d\.]+$' && ip_is_blocked "${ip}";
	then
		echo -e " \u25cc ${ip} (already dropped)"
	elif echo "${ip}" | rg -q '^[\da-fA-F:]+$' && ip6_is_blocked "${ip}";
	then
		echo -e " \u25cc ${ip} (already dropped)"
	else
		echo -e " \u25cb ${ip} (\e[4mNOT\e[0m already dropped; would drop)"
	fi
done

read -p "Continue? [y/N]: " confirm && [[ $confirm == [yY] || $confirm == [yY][eE][sS] ]] || exit 1

for ip in $(cat malicious_clients);
do
	if echo "${ip}" | rg -q '^[\d\.]+$' && ! ip_is_blocked "${ip}";
	then
		echo -e " \u25cb Dropping ${ip} (v4)..."

		set -x
		iptables -w -A INPUT -s "${ip}" -j DROP
		{ set +x; } 2>/dev/null
	elif echo "${ip}" | rg -q '^[\da-fA-F:]+$' && ! ip6_is_blocked "${ip}";
	then
		echo -e " \u25cb Dropping ${ip} (v6)..."

		set -x
		ip6tables -w -A INPUT -s "${ip}" -j DROP
		{ set +x; } 2>/dev/null
	fi
done
