# amalgam

`amalgam` is my answer to Fail2Ban.  I make no claims about whether or
not `amalgam` is better&mdash;in fact, I'll readily say it's certainly
not.  But `amalgam` is something similar to what I have always wanted
for a firewall.  `amalgam` aims to be

- _streaming_, reading inputs from standard input and standard output,
- _composable_, allowing different formations of `amalgam` instances
  to co-operate on tasks, and
- _configurable_ to the fullest extent necessary.

## (Prototype) Usage

The prototype script, `amalgam.bash` (which is symlinked by the file
called `amalgam` at the tree root) should be run with `sudo` on a
system with `rg` (ripgrep), and the `iptables` and `ipset` userspace
utilities, and on a system that has authentication logs available in
the `sshd` journald unit (more on this below).

Use this script as follows:

```console
$ sudo ./amalgam
```

That's it.

## v1 (Rust)

Starting in late 2019, I'm working on a Rust version of `amalgam` to
actually satisfy the stated goals of this project.

Using `serde_json` and lots of Rusty tricks, `amalgam` hopes to
eventually move forward as a standalone utility.

Eventually, the usage will look more like this:

```console
$ journalctl -u sshd -o json | amalgam -i- --input-format journalctl-json -c config.yml
```

This is, of course, a work in progress until this new syntax has been
established.
