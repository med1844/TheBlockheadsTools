# Server Development Notes

This guide contains useful notes for those working with or developing for server deployments of *The Blockheads*.

## `libdispatch` to run server

In Ubuntu 24.04 there's no longer `libdispatch` in the system repositories. You must manually download and install them from these addresses:

- [`libdispatch`](https://launchpad.net/ubuntu/zesty/amd64/libdispatch0/0~svn197-3.3ubuntu2)
- [`libdispatch-dev`](https://launchpad.net/ubuntu/focal/amd64/libdispatch-dev/0~svn197-3.3ubuntu2)

## Capturing server packets

You can capture network packets to or from the server for debugging purposes using the following command:

```bash
sudo tcpdump -i any -w test.pcap port 15151
```
